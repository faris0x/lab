//! Model parsing, standard form conversion, and the simplex / Big-M engine.

use mathcore::{MVal, Rat};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sense {
    Max,
    Min,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Le,
    Ge,
    Eq,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VarKind {
    Decision,
    Slack,
    Surplus,
    Artificial,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub name: String,
    pub kind: VarKind,
}

impl Header {
    pub fn latex(&self) -> String {
        var_latex(&self.name)
    }
}

#[derive(Clone, Debug)]
pub struct Constraint {
    pub coeffs: Vec<Rat>,
    pub op: Op,
    pub rhs: Rat,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub sense: Sense,
    pub vars: Vec<String>,
    pub objective: Vec<Rat>,
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, Debug)]
pub struct Tableau {
    pub headers: Vec<Header>,
    pub rows: Vec<Vec<MVal>>,
    pub rhs: Vec<MVal>,
    pub basis: Vec<usize>,
    pub obj: Vec<MVal>,
    pub obj_rhs: MVal,
}

#[derive(Clone, Debug)]
pub struct Standard {
    pub tableau: Tableau,
    pub artificial_cols: Vec<usize>,
    pub transformed: bool,
    pub objective_plain: String,
    pub objective_latex: String,
    pub constraint_plain: Vec<String>,
    pub constraint_latex: Vec<String>,
    pub objective_row_plain: String,
    pub objective_row_latex: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Optimal,
    Unbounded,
    Infeasible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    Iteration,
    Optimal,
    Unbounded,
    Infeasible,
}

#[derive(Clone, Copy, Debug)]
pub struct Ratio {
    pub row: usize,
    pub value: Rat,
}

#[derive(Clone, Copy, Debug)]
pub struct RowOp {
    pub target: usize,
    pub source: usize,
    pub factor: Rat,
}

#[derive(Clone, Debug)]
pub struct Step {
    pub kind: StepKind,
    pub tableau: Tableau,
    pub pivot: Option<(usize, usize)>,
    pub entering: Option<usize>,
    pub leaving: Option<usize>,
    pub ratios: Vec<Ratio>,
    pub row_ops: Vec<RowOp>,
    pub note_plain: Vec<String>,
    pub note_latex: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Solution {
    pub model: Model,
    pub standard: Standard,
    pub status: Status,
    pub steps: Vec<Step>,
    pub values: Vec<Rat>,
    pub optimum: Option<Rat>,
    pub iterations: usize,
    pub summary_plain: Vec<String>,
    pub summary_latex: Vec<String>,
}

pub fn var_latex(name: &str) -> String {
    let split = name
        .char_indices()
        .rev()
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(i, _)| i + 1);
    if let Some(i) = split {
        if i < name.len() {
            return format!("{}_{{{}}}", &name[..i], &name[i..]);
        }
    }
    name.to_string()
}

fn fresh(prefix: &str, used: &mut Vec<String>) -> String {
    let mut i = 1usize;
    loop {
        let candidate = format!("{prefix}{i}");
        if !used.iter().any(|u| u == &candidate) {
            used.push(candidate.clone());
            return candidate;
        }
        i += 1;
    }
}

fn flip_op(op: Op) -> Op {
    match op {
        Op::Le => Op::Ge,
        Op::Ge => Op::Le,
        Op::Eq => Op::Eq,
    }
}

pub fn build_standard(model: &Model) -> Result<Standard, String> {
    let n = model.vars.len();
    if model.objective.len() != n {
        return Err("objective length does not match the number of variables".into());
    }

    // Normalise so the RHS is non-negative.
    let mut cons: Vec<Constraint> = Vec::with_capacity(model.constraints.len());
    for c in &model.constraints {
        if c.coeffs.len() != n {
            return Err("a constraint does not match the number of variables".into());
        }
        if c.rhs.is_negative() {
            let mut coeffs = Vec::with_capacity(n);
            for v in &c.coeffs {
                coeffs.push(v.neg());
            }
            cons.push(Constraint {
                coeffs,
                op: flip_op(c.op),
                rhs: c.rhs.neg(),
            });
        } else {
            cons.push(c.clone());
        }
    }

    let mut used = model.vars.clone();
    let mut headers: Vec<Header> = model
        .vars
        .iter()
        .map(|v| Header {
            name: v.clone(),
            kind: VarKind::Decision,
        })
        .collect();

    // (slack col, surplus col, artificial col) per constraint
    let mut added: Vec<(Option<usize>, Option<usize>, Option<usize>)> = Vec::new();
    for c in &cons {
        match c.op {
            Op::Le => {
                let idx = headers.len();
                let name = fresh("s", &mut used);
                headers.push(Header {
                    name,
                    kind: VarKind::Slack,
                });
                added.push((Some(idx), None, None));
            }
            Op::Ge => {
                let si = headers.len();
                let sname = fresh("e", &mut used);
                headers.push(Header {
                    name: sname,
                    kind: VarKind::Surplus,
                });
                let ai = headers.len();
                let aname = fresh("a", &mut used);
                headers.push(Header {
                    name: aname,
                    kind: VarKind::Artificial,
                });
                added.push((None, Some(si), Some(ai)));
            }
            Op::Eq => {
                let ai = headers.len();
                let aname = fresh("a", &mut used);
                headers.push(Header {
                    name: aname,
                    kind: VarKind::Artificial,
                });
                added.push((None, None, Some(ai)));
            }
        }
    }

    let width = headers.len();
    let mut rows: Vec<Vec<MVal>> = Vec::with_capacity(cons.len());
    let mut rhs: Vec<MVal> = Vec::with_capacity(cons.len());
    let mut basis: Vec<usize> = Vec::with_capacity(cons.len());

    for (i, c) in cons.iter().enumerate() {
        let mut row = vec![MVal::ZERO; width];
        for (j, v) in c.coeffs.iter().enumerate() {
            row[j] = MVal::rat(*v);
        }
        if let (Some(s), _, _) = added[i] {
            row[s] = MVal::ONE;
            basis.push(s);
        }
        if let (_, Some(s), _) = added[i] {
            row[s] = MVal::rat(Rat::int(-1));
        }
        if let (_, _, Some(a)) = added[i] {
            row[a] = MVal::ONE;
            basis.push(a);
        }
        rows.push(row);
        rhs.push(MVal::rat(c.rhs));
    }

    let transformed = model.sense == Sense::Min;
    let solved_objective: Vec<Rat> = if transformed {
        model.objective.iter().map(|v| v.neg()).collect()
    } else {
        model.objective.clone()
    };

    // Z - c'x + M a = 0
    let mut obj = vec![MVal::ZERO; width];
    for (j, c) in solved_objective.iter().enumerate() {
        obj[j] = MVal::rat(c.neg());
    }
    for (j, h) in headers.iter().enumerate() {
        if h.kind == VarKind::Artificial {
            obj[j] = MVal {
                a: Rat::ZERO,
                b: Rat::ONE,
            };
        }
    }

    // Canonicalise the objective row against the artificial basis:
    // obj <- obj - M * row for every basic artificial variable.
    let obj_raw = obj.clone();
    let mut obj_rhs = MVal::ZERO;
    for i in 0..rows.len() {
        if headers[basis[i]].kind == VarKind::Artificial {
            for j in 0..width {
                let r = rows[i][j].a;
                if !r.is_zero() {
                    obj[j] = obj[j].sub(MVal { a: Rat::ZERO, b: r })?;
                }
            }
            let r = rhs[i].a;
            if !r.is_zero() {
                obj_rhs = obj_rhs.sub(MVal { a: Rat::ZERO, b: r })?;
            }
        }
    }

    let artificial_cols: Vec<usize> = headers
        .iter()
        .enumerate()
        .filter(|(_, h)| h.kind == VarKind::Artificial)
        .map(|(i, _)| i)
        .collect();

    let tableau = Tableau {
        headers: headers.clone(),
        rows: rows.clone(),
        rhs: rhs.clone(),
        basis: basis.clone(),
        obj: obj.clone(),
        obj_rhs,
    };

    let objective_plain = if transformed {
        format!(
            "Minimise C = {} (solved as Maximise Z = -C = {})",
            linear_plain(&model.objective, &model.vars),
            linear_plain(&solved_objective, &model.vars)
        )
    } else {
        format!(
            "Maximise Z = {}",
            linear_plain(&model.objective, &model.vars)
        )
    };
    let objective_latex = if transformed {
        format!(
            "\\text{{Minimise }} C = {} \\quad\\text{{(solved as Maximise }} Z = -C = {}\\text{{)}}",
            linear_latex(&model.objective, &model.vars),
            linear_latex(&solved_objective, &model.vars)
        )
    } else {
        format!(
            "\\text{{Maximise }} Z = {}",
            linear_latex(&model.objective, &model.vars)
        )
    };

    let mut constraint_plain = Vec::new();
    let mut constraint_latex = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let lhs = expr_plain(row, &headers);
        let lhs_tex = expr_latex(row, &headers);
        let r = rhs[i].a;
        constraint_plain.push(format!("{lhs} = {}", r.plain()));
        constraint_latex.push(format!("{lhs_tex} = {}", r.latex()));
    }

    let obj_lhs = expr_plain(&obj_raw, &headers);
    let obj_lhs_tex = expr_latex(&obj_raw, &headers);
    let objective_row_plain = with_prefix_plain("Z", &obj_lhs) + " = 0";
    let objective_row_latex = with_prefix_latex("Z", &obj_lhs_tex) + " = 0";

    Ok(Standard {
        tableau,
        artificial_cols,
        transformed,
        objective_plain,
        objective_latex,
        constraint_plain,
        constraint_latex,
        objective_row_plain,
        objective_row_latex,
    })
}

fn choose_entering(obj: &[MVal]) -> Option<usize> {
    let mut best: Option<(usize, MVal)> = None;
    for (j, v) in obj.iter().enumerate() {
        if v.is_negative() {
            match best {
                None => best = Some((j, *v)),
                Some((bj, bv)) => {
                    let ord = v.cmp_m(bv);
                    if ord == Ordering::Less || (ord == Ordering::Equal && j < bj) {
                        best = Some((j, *v));
                    }
                }
            }
        }
    }
    best.map(|(j, _)| j)
}

fn ratio_test(tab: &Tableau, col: usize) -> Result<Vec<Ratio>, String> {
    let mut out = Vec::new();
    for (i, row) in tab.rows.iter().enumerate() {
        let a = row[col];
        if a.b.is_zero() && a.a.is_positive() {
            out.push(Ratio {
                row: i,
                value: tab.rhs[i].a.div(a.a)?,
            });
        }
    }
    Ok(out)
}

fn best_ratio(ratios: &[Ratio]) -> Option<Ratio> {
    let mut best: Option<Ratio> = None;
    for r in ratios {
        match best {
            None => best = Some(*r),
            Some(b) => {
                let ord = r.value.cmp_rat(b.value);
                if ord == Ordering::Less || (ord == Ordering::Equal && r.row < b.row) {
                    best = Some(*r);
                }
            }
        }
    }
    best
}

fn pivot(tab: &mut Tableau, r: usize, c: usize) -> Result<(), String> {
    let pv = tab.rows[r][c];
    if !(pv.b.is_zero() && pv.a.is_positive()) {
        return Err("internal error: pivot is not positive".into());
    }
    let pr = pv.a;
    let width = tab.headers.len();

    for cell in tab.rows[r].iter_mut().take(width) {
        *cell = cell.div_rat(pr)?;
    }
    tab.rhs[r] = tab.rhs[r].div_rat(pr)?;

    for i in 0..tab.rows.len() {
        if i == r {
            continue;
        }
        let f = tab.rows[i][c];
        if f.is_zero() {
            continue;
        }
        if !f.b.is_zero() {
            return Err("internal error: non-rational constraint coefficient".into());
        }
        for j in 0..width {
            let above = tab.rows[r][j];
            tab.rows[i][j] = tab.rows[i][j].sub(above.mul_rat(f.a)?)?;
        }
        tab.rhs[i] = tab.rhs[i].sub(tab.rhs[r].mul_rat(f.a)?)?;
        tab.rows[i][c] = MVal::ZERO;
    }

    let f = tab.obj[c];
    if !f.is_zero() {
        for (objective, row_cell) in tab.obj.iter_mut().zip(tab.rows[r].iter()).take(width) {
            *objective = objective.sub(f.mul_rat(row_cell.a)?)?;
        }
        let rhs_cell = tab.rhs[r].a;
        tab.obj_rhs = tab.obj_rhs.sub(f.mul_rat(rhs_cell)?)?;
        tab.obj[c] = MVal::ZERO;
    }

    tab.basis[r] = c;
    Ok(())
}

fn row_ops_for(tab: &Tableau, r: usize, c: usize) -> Result<Vec<RowOp>, String> {
    let mut ops = Vec::new();
    for (i, row) in tab.rows.iter().enumerate() {
        if i == r {
            continue;
        }
        let f = row[c];
        if !f.is_zero() {
            if !f.b.is_zero() {
                return Err("internal error: M in a constraint row".into());
            }
            ops.push(RowOp {
                target: i,
                source: r,
                factor: f.a,
            });
        }
    }
    Ok(ops)
}

fn artificial_at_positive(tab: &Tableau, artificial_cols: &[usize]) -> Option<usize> {
    for (i, &b) in tab.basis.iter().enumerate() {
        if artificial_cols.contains(&b) && !tab.rhs[i].is_zero() {
            return Some(i);
        }
    }
    None
}

pub fn solve(model: Model) -> Result<Solution, String> {
    if model.vars.is_empty() {
        return Err("add at least one variable".into());
    }
    if model.constraints.is_empty() {
        return Err("add at least one constraint".into());
    }
    let standard = build_standard(&model)?;
    let mut tab = standard.tableau.clone();
    let mut steps: Vec<Step> = Vec::new();
    let mut iterations = 0usize;
    let mut status = Status::Optimal;

    loop {
        if iterations > 300 {
            return Err("iteration limit reached; the problem may be cycling".into());
        }
        match choose_entering(&tab.obj) {
            None => {
                if artificial_at_positive(&tab, &standard.artificial_cols).is_some() {
                    status = Status::Infeasible;
                    steps.push(Step {
                        kind: StepKind::Infeasible,
                        tableau: tab.clone(),
                        pivot: None,
                        entering: None,
                        leaving: None,
                        ratios: Vec::new(),
                        row_ops: Vec::new(),
                        note_plain: vec![
                            "An artificial variable is still positive, so the constraints cannot all be satisfied. The problem is infeasible.".into(),
                        ],
                        note_latex: vec![
                            "An artificial variable is still positive, so the constraints cannot all be satisfied. The problem is infeasible.".into(),
                        ],
                    });
                } else {
                    steps.push(Step {
                        kind: StepKind::Optimal,
                        tableau: tab.clone(),
                        pivot: None,
                        entering: None,
                        leaving: None,
                        ratios: Vec::new(),
                        row_ops: Vec::new(),
                        note_plain: vec![
                            "No negative entries remain in the objective row, so this tableau is optimal.".into(),
                        ],
                        note_latex: vec![
                            "No negative entries remain in the objective row, so this tableau is optimal.".into(),
                        ],
                    });
                }
                break;
            }
            Some(col) => {
                let ratios = ratio_test(&tab, col)?;
                if ratios.is_empty() {
                    status = Status::Unbounded;
                    let name = tab.headers[col].name.clone();
                    let tex = tab.headers[col].latex();
                    steps.push(Step {
                        kind: StepKind::Unbounded,
                        tableau: tab.clone(),
                        pivot: None,
                        entering: Some(col),
                        leaving: None,
                        ratios: Vec::new(),
                        row_ops: Vec::new(),
                        note_plain: vec![format!(
                            "The entering variable {name} has no positive coefficient in any constraint row, so it can increase without limit. The objective is unbounded."
                        )],
                        note_latex: vec![format!(
                            "The entering variable ${tex}$ has no positive coefficient in any constraint row, so it can increase without limit. The objective is unbounded."
                        )],
                    });
                    break;
                }
                let best = best_ratio(&ratios).ok_or("internal error: no ratio")?;
                let row = best.row;
                let entering = col;
                let leaving = tab.basis[row];
                let ops = row_ops_for(&tab, row, col)?;

                let mut note_plain = Vec::new();
                let mut note_latex = Vec::new();
                let evar = tab.headers[entering].name.clone();
                let evar_tex = tab.headers[entering].latex();
                let ecoef = tab.obj[entering];
                note_plain.push(format!(
                    "The most negative entry in the objective row is {}, in the {} column, so {} enters the basis.",
                    ecoef.plain(),
                    evar,
                    evar
                ));
                note_latex.push(format!(
                    "The most negative entry in the objective row is ${}$, in the ${}$ column, so ${}$ enters the basis.",
                    ecoef.latex(),
                    evar_tex,
                    evar_tex
                ));

                let mut ratio_bits = Vec::new();
                let mut ratio_bits_tex = Vec::new();
                for r in &ratios {
                    let cell = tab.rows[r.row][entering].a;
                    let rhs = tab.rhs[r.row].a;
                    let label = format!("R{}", r.row + 1);
                    ratio_bits.push(format!(
                        "{label}: {} ÷ {} = {}",
                        rhs.plain(),
                        cell.plain(),
                        r.value.plain()
                    ));
                    ratio_bits_tex.push(format!(
                        "R_{{{}}}: {} \\div {} = {}",
                        r.row + 1,
                        rhs.latex(),
                        cell.latex(),
                        r.value.latex()
                    ));
                }
                let lvar = tab.headers[leaving].name.clone();
                let lvar_tex = tab.headers[leaving].latex();
                note_plain.push(format!(
                    "Ratio test: {}. The smallest ratio is {} in row {}, so {} leaves the basis.",
                    ratio_bits.join("; "),
                    best.value.plain(),
                    row + 1,
                    lvar
                ));
                note_latex.push(format!(
                    "Ratio test: ${}$. The smallest ratio is ${}$ in row {}, so ${}$ leaves the basis.",
                    ratio_bits_tex.join(";\\; "),
                    best.value.latex(),
                    row + 1,
                    lvar_tex
                ));

                if !ops.is_empty() {
                    let mut bits = Vec::new();
                    let mut bits_tex = Vec::new();
                    for op in &ops {
                        bits.push(row_op_plain(op));
                        bits_tex.push(row_op_latex(op));
                    }
                    note_plain.push(format!(
                        "Pivot on the {} in row {}, column {}: {}.",
                        tab.rows[row][entering].a.plain(),
                        row + 1,
                        evar,
                        bits.join("; ")
                    ));
                    note_latex.push(format!(
                        "Pivot on the ${}$ in row {}, column ${}$: ${}$.",
                        tab.rows[row][entering].a.latex(),
                        row + 1,
                        evar_tex,
                        bits_tex.join(";\\; ")
                    ));
                }

                steps.push(Step {
                    kind: StepKind::Iteration,
                    tableau: tab.clone(),
                    pivot: Some((row, entering)),
                    entering: Some(entering),
                    leaving: Some(leaving),
                    ratios: ratios.clone(),
                    row_ops: ops,
                    note_plain,
                    note_latex,
                });

                pivot(&mut tab, row, entering)?;
                iterations += 1;
            }
        }
    }

    let mut values = vec![Rat::ZERO; model.vars.len()];
    for (i, &col) in tab.basis.iter().enumerate() {
        if col < model.vars.len() && !tab.rhs[i].b.is_zero() {
            return Err("internal error: M term in the solution".into());
        }
        if col < model.vars.len() {
            values[col] = tab.rhs[i].a;
        }
    }

    let optimum = match status {
        Status::Optimal => {
            let z = tab.obj_rhs;
            if !z.b.is_zero() {
                return Err("internal error: M term in the objective".into());
            }
            Some(if model.sense == Sense::Min {
                z.a.neg()
            } else {
                z.a
            })
        }
        _ => None,
    };

    let mut summary_plain = Vec::new();
    let mut summary_latex = Vec::new();
    match status {
        Status::Optimal => {
            let label = if model.sense == Sense::Min { "C" } else { "Z" };
            let value = optimum.unwrap();
            let mut bits = Vec::new();
            let mut bits_tex = Vec::new();
            for (i, v) in values.iter().enumerate() {
                bits.push(format!("{} = {}", model.vars[i], v.plain()));
                bits_tex.push(format!("{} = {}", var_latex(&model.vars[i]), v.latex()));
            }
            summary_plain.push(format!(
                "The optimal value is {label} = {} at {}.",
                value.plain(),
                bits.join(", ")
            ));
            summary_latex.push(format!(
                "The optimal value is ${label} = {}$ at ${}$.",
                value.latex(),
                bits_tex.join(",\\; ")
            ));
            if model.sense == Sense::Min {
                summary_plain.push(format!(
                    "The tableau was solved as a maximisation of Z = -C, and the reported minimum is C = -Z = {}.",
                    value.plain()
                ));
                summary_latex.push(format!(
                    "The tableau was solved as a maximisation of $Z = -C$, and the reported minimum is $C = -Z = {}$.",
                    value.latex()
                ));
            }
        }
        Status::Infeasible => {
            summary_plain
                .push("The problem is infeasible: no point satisfies all constraints.".into());
            summary_latex
                .push("The problem is infeasible: no point satisfies all constraints.".into());
        }
        Status::Unbounded => {
            summary_plain.push("The objective is unbounded: it can be made arbitrarily large (or small) while satisfying the constraints.".into());
            summary_latex.push("The objective is unbounded: it can be made arbitrarily large (or small) while satisfying the constraints.".into());
        }
    }

    Ok(Solution {
        model,
        standard,
        status,
        steps,
        values,
        optimum,
        iterations,
        summary_plain,
        summary_latex,
    })
}

pub fn row_op_plain(op: &RowOp) -> String {
    let t = op.target + 1;
    let s = op.source + 1;
    if op.factor == Rat::ONE {
        format!("R{t} ← R{t} − R{s}")
    } else if op.factor == Rat::int(-1) {
        format!("R{t} ← R{t} + R{s}")
    } else if op.factor.is_negative() {
        format!("R{t} ← R{t} + {}R{s}", op.factor.abs().plain())
    } else {
        format!("R{t} ← R{t} − {}R{s}", op.factor.plain())
    }
}

pub fn row_op_latex(op: &RowOp) -> String {
    let t = op.target + 1;
    let s = op.source + 1;
    if op.factor == Rat::ONE {
        format!("R_{{{t}}} \\leftarrow R_{{{t}}} - R_{{{s}}}")
    } else if op.factor == Rat::int(-1) {
        format!("R_{{{t}}} \\leftarrow R_{{{t}}} + R_{{{s}}}")
    } else if op.factor.is_negative() {
        format!(
            "R_{{{t}}} \\leftarrow R_{{{t}}} + {}R_{{{s}}}",
            op.factor.abs().latex()
        )
    } else {
        format!(
            "R_{{{t}}} \\leftarrow R_{{{t}}} - {}R_{{{s}}}",
            op.factor.latex()
        )
    }
}

pub fn linear_plain(coeffs: &[Rat], vars: &[String]) -> String {
    let mut out = String::new();
    for (i, c) in coeffs.iter().enumerate() {
        if c.is_zero() {
            continue;
        }
        let neg = c.is_negative();
        let mag = if neg { c.abs() } else { *c };
        if out.is_empty() {
            if neg {
                out.push('-');
            }
        } else {
            out.push_str(if neg { " - " } else { " + " });
        }
        let name = &vars[i];
        if mag == Rat::ONE {
            out.push_str(name);
        } else if mag.d == 1 {
            out.push_str(&mag.plain());
            out.push_str(name);
        } else {
            out.push('(');
            out.push_str(&mag.plain());
            out.push(')');
            out.push_str(name);
        }
    }
    if out.is_empty() {
        out.push('0');
    }
    out
}

pub fn linear_latex(coeffs: &[Rat], vars: &[String]) -> String {
    let mut out = String::new();
    for (i, c) in coeffs.iter().enumerate() {
        if c.is_zero() {
            continue;
        }
        let neg = c.is_negative();
        let mag = if neg { c.abs() } else { *c };
        if out.is_empty() {
            if neg {
                out.push('-');
            }
        } else {
            out.push_str(if neg { " - " } else { " + " });
        }
        let name = var_latex(&vars[i]);
        if mag == Rat::ONE {
            out.push_str(&name);
        } else {
            out.push_str(&mag.latex());
            out.push_str(&name);
        }
    }
    if out.is_empty() {
        out.push('0');
    }
    out
}

pub fn expr_plain(coeffs: &[MVal], headers: &[Header]) -> String {
    let mut out = String::new();
    for (j, c) in coeffs.iter().enumerate() {
        if c.is_zero() {
            continue;
        }
        let neg = c.is_negative();
        let mag = if neg { c.neg() } else { *c };
        if out.is_empty() {
            if neg {
                out.push('-');
            }
        } else {
            out.push_str(if neg { " - " } else { " + " });
        }
        let name = &headers[j].name;
        if mag == MVal::ONE {
            out.push_str(name);
        } else {
            let cs = mag.abs_plain();
            if (!mag.b.is_zero() && !mag.a.is_zero()) || (mag.b.is_zero() && mag.a.d != 1) {
                out.push('(');
                out.push_str(&cs);
                out.push(')');
                out.push_str(name);
            } else if !mag.b.is_zero() {
                out.push_str(&cs);
                out.push('·');
                out.push_str(name);
            } else {
                out.push_str(&cs);
                out.push_str(name);
            }
        }
    }
    if out.is_empty() {
        out.push('0');
    }
    out
}

pub fn expr_latex(coeffs: &[MVal], headers: &[Header]) -> String {
    let mut out = String::new();
    for (j, c) in coeffs.iter().enumerate() {
        if c.is_zero() {
            continue;
        }
        let neg = c.is_negative();
        let mag = if neg { c.neg() } else { *c };
        if out.is_empty() {
            if neg {
                out.push('-');
            }
        } else {
            out.push_str(if neg { " - " } else { " + " });
        }
        let name = headers[j].latex();
        if mag == MVal::ONE {
            out.push_str(&name);
        } else {
            let cs = mag.abs_latex();
            if !mag.b.is_zero() && !mag.a.is_zero() {
                out.push_str("\\left(");
                out.push_str(&cs);
                out.push_str("\\right)");
                out.push_str(&name);
            } else if !mag.b.is_zero() {
                out.push_str(&cs);
                out.push_str("\\,");
                out.push_str(&name);
            } else {
                out.push_str(&cs);
                out.push_str(&name);
            }
        }
    }
    if out.is_empty() {
        out.push('0');
    }
    out
}

fn with_prefix_plain(prefix: &str, expr: &str) -> String {
    if let Some(rest) = expr.strip_prefix('-') {
        format!("{prefix} - {rest}")
    } else if expr == "0" {
        prefix.to_string()
    } else {
        format!("{prefix} + {expr}")
    }
}

fn with_prefix_latex(prefix: &str, expr: &str) -> String {
    if let Some(rest) = expr.strip_prefix('-') {
        format!("{prefix} - {rest}")
    } else if expr == "0" {
        prefix.to_string()
    } else {
        format!("{prefix} + {expr}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn max_model() -> Model {
        Model {
            sense: Sense::Max,
            vars: vec!["x".into(), "y".into()],
            objective: vec![Rat::int(3), Rat::int(2)],
            constraints: vec![
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(1)],
                    op: Op::Le,
                    rhs: Rat::int(4),
                },
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(3)],
                    op: Op::Le,
                    rhs: Rat::int(6),
                },
            ],
        }
    }

    fn min_big_m_model() -> Model {
        Model {
            sense: Sense::Min,
            vars: vec!["x".into(), "y".into()],
            objective: vec![Rat::int(2), Rat::int(3)],
            constraints: vec![
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(1)],
                    op: Op::Ge,
                    rhs: Rat::int(4),
                },
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(3)],
                    op: Op::Ge,
                    rhs: Rat::int(6),
                },
            ],
        }
    }

    #[test]
    fn solves_a_small_maximisation() {
        let sol = solve(max_model()).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(12)));
        assert_eq!(sol.values, vec![Rat::int(4), Rat::int(0)]);
        assert!(!sol.steps.is_empty());
        assert_eq!(sol.steps.last().unwrap().kind, StepKind::Optimal);
    }

    #[test]
    fn solves_a_minimisation_with_big_m() {
        let sol = solve(min_big_m_model()).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(9)));
        assert_eq!(sol.values, vec![Rat::int(3), Rat::int(1)]);
        assert!(sol.standard.transformed);
        assert!(!sol.standard.artificial_cols.is_empty());
    }

    #[test]
    fn detects_infeasibility() {
        let model = Model {
            sense: Sense::Max,
            vars: vec!["x".into(), "y".into()],
            objective: vec![Rat::int(1), Rat::int(1)],
            constraints: vec![
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(1)],
                    op: Op::Le,
                    rhs: Rat::int(1),
                },
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(1)],
                    op: Op::Ge,
                    rhs: Rat::int(2),
                },
            ],
        };
        let sol = solve(model).unwrap();
        assert_eq!(sol.status, Status::Infeasible);
        assert!(sol.optimum.is_none());
    }

    #[test]
    fn detects_unboundedness() {
        let model = Model {
            sense: Sense::Max,
            vars: vec!["x".into(), "y".into()],
            objective: vec![Rat::int(1), Rat::int(1)],
            constraints: vec![Constraint {
                coeffs: vec![Rat::int(1), Rat::int(-1)],
                op: Op::Le,
                rhs: Rat::int(1),
            }],
        };
        let sol = solve(model).unwrap();
        assert_eq!(sol.status, Status::Unbounded);
    }

    #[test]
    fn solves_a_maximisation_with_big_m() {
        let model = Model {
            sense: Sense::Max,
            vars: vec!["x".into(), "y".into()],
            objective: vec![Rat::int(1), Rat::int(1)],
            constraints: vec![
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(1)],
                    op: Op::Ge,
                    rhs: Rat::int(2),
                },
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(0)],
                    op: Op::Le,
                    rhs: Rat::int(3),
                },
                Constraint {
                    coeffs: vec![Rat::int(0), Rat::int(1)],
                    op: Op::Le,
                    rhs: Rat::int(3),
                },
            ],
        };
        let sol = solve(model).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(6)));
        assert_eq!(sol.values, vec![Rat::int(3), Rat::int(3)]);
    }

    #[test]
    fn handles_an_equality_constraint_with_big_m() {
        let model = Model {
            sense: Sense::Max,
            vars: vec!["x".into(), "y".into()],
            objective: vec![Rat::int(3), Rat::int(2)],
            constraints: vec![
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(1)],
                    op: Op::Eq,
                    rhs: Rat::int(4),
                },
                Constraint {
                    coeffs: vec![Rat::int(1), Rat::int(0)],
                    op: Op::Le,
                    rhs: Rat::int(3),
                },
            ],
        };
        let sol = solve(model).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(11)));
        assert_eq!(sol.values, vec![Rat::int(3), Rat::int(1)]);
    }

    #[test]
    fn normalises_a_negative_right_hand_side() {
        let model = Model {
            sense: Sense::Max,
            vars: vec!["x".into()],
            objective: vec![Rat::int(1)],
            constraints: vec![
                Constraint {
                    coeffs: vec![Rat::int(-1)],
                    op: Op::Le,
                    rhs: Rat::int(-2),
                },
                Constraint {
                    coeffs: vec![Rat::int(1)],
                    op: Op::Le,
                    rhs: Rat::int(5),
                },
            ],
        };
        let sol = solve(model).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(5)));
        assert_eq!(sol.values, vec![Rat::int(5)]);
    }

    #[test]
    fn handles_fractions_in_the_tableau() {
        let model = Model {
            sense: Sense::Max,
            vars: vec!["x".into()],
            objective: vec![Rat::int(1)],
            constraints: vec![Constraint {
                coeffs: vec![Rat::new(1, 3).unwrap()],
                op: Op::Le,
                rhs: Rat::int(1),
            }],
        };
        let sol = solve(model).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(3)));
        assert_eq!(sol.values, vec![Rat::int(3)]);
    }
}
