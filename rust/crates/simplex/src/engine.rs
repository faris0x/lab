//! Model parsing, standard form conversion, and the simplex / Big-M engine.

use mathcore::{MVal, Rat};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sense {
    Max,
    Min,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    BigM,
    TwoStage,
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
    Start,
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
    pub phase: u8,
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
pub enum Source {
    Model {
        model: Box<Model>,
        standard: Box<Standard>,
        method: Method,
    },
    Tableau {
        start: Tableau,
        decision_cols: Vec<usize>,
        objective_label: String,
        report_sign: i8,
    },
}

#[derive(Clone, Debug)]
pub struct Solution {
    pub source: Source,
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

fn choose_entering(obj: &[MVal], forbidden: &[usize]) -> Option<usize> {
    let mut best: Option<(usize, MVal)> = None;
    for (j, v) in obj.iter().enumerate() {
        if forbidden.contains(&j) {
            continue;
        }
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

pub struct ActionResult {
    pub status: Status,
    pub steps: Vec<Step>,
    pub iterations: usize,
    pub tableau: Tableau,
}

fn iterate(
    mut tab: Tableau,
    artificial_cols: &[usize],
    max_iterations: usize,
    phase: u8,
) -> Result<ActionResult, String> {
    let mut steps: Vec<Step> = Vec::new();
    let mut iterations = 0usize;
    let mut status = Status::Optimal;

    loop {
        if iterations > max_iterations {
            return Err("iteration limit reached; the problem may be cycling".into());
        }
        match choose_entering(&tab.obj, artificial_cols) {
            None => {
                if artificial_at_positive(&tab, artificial_cols).is_some() {
                    status = Status::Infeasible;
                    steps.push(Step {
                        kind: StepKind::Infeasible,
                        phase,
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
                        phase,
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
                        phase,
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
                    phase,
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

    Ok(ActionResult {
        status,
        steps,
        iterations,
        tableau: tab,
    })
}

fn extract_values(tab: &Tableau, decision_cols: &[usize]) -> Result<Vec<Rat>, String> {
    let mut values = vec![Rat::ZERO; decision_cols.len()];
    for (i, &col) in tab.basis.iter().enumerate() {
        if let Some(position) = decision_cols.iter().position(|&candidate| candidate == col) {
            if !tab.rhs[i].b.is_zero() {
                return Err("internal error: M term in the solution".into());
            }
            values[position] = tab.rhs[i].a;
        }
    }
    Ok(values)
}

fn extract_optimum(tab: &Tableau, report_sign: i8) -> Result<Option<Rat>, String> {
    let z = tab.obj_rhs;
    if !z.b.is_zero() {
        return Err("internal error: M term in the objective".into());
    }
    Ok(Some(if report_sign < 0 { z.a.neg() } else { z.a }))
}

fn solved_objective(model: &Model) -> Vec<Rat> {
    if model.sense == Sense::Min {
        model.objective.iter().map(|value| value.neg()).collect()
    } else {
        model.objective.clone()
    }
}

fn canonicalize_objective(
    tab: &Tableau,
    obj: &mut [MVal],
    obj_rhs: &mut MVal,
) -> Result<(), String> {
    for (i, &basic) in tab.basis.iter().enumerate() {
        let factor = obj[basic];
        if factor.is_zero() {
            continue;
        }
        if !factor.b.is_zero() {
            return Err("internal error: M term while canonicalising the objective".into());
        }
        for (j, cell) in tab.rows[i].iter().enumerate() {
            obj[j] = obj[j].sub(cell.mul_rat(factor.a)?)?;
        }
        *obj_rhs = obj_rhs.sub(tab.rhs[i].mul_rat(factor.a)?)?;
    }
    Ok(())
}

fn phase_one_objective(
    tab: &Tableau,
    artificial_cols: &[usize],
) -> Result<(Vec<MVal>, MVal), String> {
    let mut obj = vec![MVal::ZERO; tab.headers.len()];
    for &column in artificial_cols {
        obj[column] = MVal::ONE;
    }
    let mut obj_rhs = MVal::ZERO;
    canonicalize_objective(tab, &mut obj, &mut obj_rhs)?;
    Ok((obj, obj_rhs))
}

fn phase_two_objective(
    tab: &Tableau,
    solved_objective: &[Rat],
) -> Result<(Vec<MVal>, MVal), String> {
    let mut obj = vec![MVal::ZERO; tab.headers.len()];
    for (column, coefficient) in solved_objective.iter().enumerate() {
        obj[column] = MVal::rat(coefficient.neg());
    }
    let mut obj_rhs = MVal::ZERO;
    canonicalize_objective(tab, &mut obj, &mut obj_rhs)?;
    Ok((obj, obj_rhs))
}

#[cfg(test)]
fn solve(model: Model) -> Result<Solution, String> {
    solve_with_method(model, Method::BigM)
}

pub fn solve_with_method(model: Model, method: Method) -> Result<Solution, String> {
    if model.vars.is_empty() {
        return Err("add at least one variable".into());
    }
    if model.constraints.is_empty() {
        return Err("add at least one constraint".into());
    }
    let mut standard = build_standard(&model)?;
    let decision_cols: Vec<usize> = (0..model.vars.len()).collect();
    let report_sign = if model.sense == Sense::Min { -1 } else { 1 };
    let mut phase_notes_plain = Vec::new();
    let mut phase_notes_latex = Vec::new();

    let result = match method {
        Method::BigM => iterate(standard.tableau.clone(), &standard.artificial_cols, 300, 0)?,
        Method::TwoStage => {
            let mut total_iterations = 0usize;
            let mut steps = Vec::new();
            let mut tableau = standard.tableau.clone();
            let phase = if standard.artificial_cols.is_empty() {
                0
            } else {
                let mut phase_one = standard.tableau.clone();
                let (obj, obj_rhs) = phase_one_objective(&phase_one, &standard.artificial_cols)?;
                standard.objective_row_plain =
                    with_prefix_plain("W", &expr_plain(&obj, &phase_one.headers)) + " = 0";
                standard.objective_row_latex =
                    with_prefix_latex("W", &expr_latex(&obj, &phase_one.headers)) + " = 0";
                phase_one.obj = obj;
                phase_one.obj_rhs = obj_rhs;
                let first = iterate(phase_one, &standard.artificial_cols, 300, 1)?;
                total_iterations += first.iterations;
                if first.status != Status::Optimal || first.tableau.obj_rhs.a.is_negative() {
                    if first.status == Status::Unbounded {
                        return Err("phase 1 did not reach a finite optimum".into());
                    }
                    let values = extract_values(&first.tableau, &decision_cols)
                        .unwrap_or_else(|_| vec![Rat::ZERO; decision_cols.len()]);
                    return Ok(Solution {
                        source: Source::Model {
                            model: Box::new(model),
                            standard: Box::new(standard),
                            method,
                        },
                        status: Status::Infeasible,
                        steps: first.steps,
                        values,
                        optimum: None,
                        iterations: first.iterations,
                        summary_plain: vec![
                            "Phase 1 left an artificial variable positive, so the problem is infeasible.".into(),
                        ],
                        summary_latex: vec![
                            "Phase 1 left an artificial variable positive, so the problem is infeasible.".into(),
                        ],
                    });
                }
                phase_notes_plain.push(
                    "Phase 1 removed the artificial variables and produced a feasible basis (W = 0).".into(),
                );
                phase_notes_latex.push(
                    "Phase 1 removed the artificial variables and produced a feasible basis ($W = 0$).".into(),
                );
                steps = first.steps;
                tableau = first.tableau;
                2
            };

            let solved = solved_objective(&model);
            let (obj, obj_rhs) = phase_two_objective(&tableau, &solved)?;
            tableau.obj = obj;
            tableau.obj_rhs = obj_rhs;
            let second = iterate(tableau, &standard.artificial_cols, 300, phase)?;
            total_iterations += second.iterations;
            steps.extend(second.steps);
            ActionResult {
                status: second.status,
                steps,
                iterations: total_iterations,
                tableau: second.tableau,
            }
        }
    };

    let values = extract_values(&result.tableau, &decision_cols)?;
    let optimum = match result.status {
        Status::Optimal => extract_optimum(&result.tableau, report_sign)?,
        _ => None,
    };

    let mut summary_plain = phase_notes_plain;
    let mut summary_latex = phase_notes_latex;
    match result.status {
        Status::Optimal => {
            let label = if model.sense == Sense::Min { "C" } else { "Z" };
            let value = optimum.unwrap_or(Rat::ZERO);
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
        source: Source::Model {
            model: Box::new(model),
            standard: Box::new(standard),
            method,
        },
        status: result.status,
        steps: result.steps,
        values,
        optimum,
        iterations: result.iterations,
        summary_plain,
        summary_latex,
    })
}

pub struct FinishOptions {
    pub decision_cols: Vec<usize>,
    pub artificial_cols: Vec<usize>,
    pub objective_label: String,
    pub report_sign: i8,
}

fn validate_tableau(tab: &Tableau, options: &FinishOptions) -> Result<(), String> {
    let width = tab.headers.len();
    let rows = tab.rows.len();
    if rows == 0 {
        return Err("add at least one tableau row".into());
    }
    if rows > 10 {
        return Err("use at most ten tableau rows".into());
    }
    if width == 0 {
        return Err("add at least one tableau column".into());
    }
    if width > 16 {
        return Err("use at most sixteen tableau columns".into());
    }
    if tab.rhs.len() != rows {
        return Err("the right-hand side needs one value per row".into());
    }
    if tab.basis.len() != rows {
        return Err("the basis needs one variable per row".into());
    }
    if tab.obj.len() != width {
        return Err("the objective row needs one value per column".into());
    }
    if options.objective_label.trim().is_empty() {
        return Err("enter an objective label".into());
    }

    for (i, header) in tab.headers.iter().enumerate() {
        if header.name.trim().is_empty() {
            return Err(format!("column {} needs a name", i + 1));
        }
        for other in i + 1..width {
            if tab.headers[other].name == header.name {
                return Err(format!("duplicate column name '{}'", header.name));
            }
        }
    }
    for row in &tab.rows {
        if row.len() != width {
            return Err("every tableau row needs one value per column".into());
        }
    }

    for (r, &basic) in tab.basis.iter().enumerate() {
        if basic >= width {
            return Err(format!("row {} has an invalid basic variable", r + 1));
        }
        if tab.basis[..r].contains(&basic) {
            return Err(format!(
                "'{}' is basic in more than one row",
                tab.headers[basic].name
            ));
        }
    }
    for (r, &basic) in tab.basis.iter().enumerate() {
        if tab.rows[r][basic] != MVal::ONE {
            return Err(format!(
                "row {}: the basic cell for '{}' must be 1",
                r + 1,
                tab.headers[basic].name
            ));
        }
        for (i, row) in tab.rows.iter().enumerate() {
            if i != r && !row[basic].is_zero() {
                return Err(format!(
                    "column '{}' is not a unit column; row {} must be zero there",
                    tab.headers[basic].name,
                    i + 1
                ));
            }
        }
    }

    for (index, &column) in options.decision_cols.iter().enumerate() {
        if column >= width {
            return Err("a decision column is out of range".into());
        }
        if options.decision_cols[..index].contains(&column) {
            return Err("decision columns cannot repeat".into());
        }
    }
    for (index, &column) in options.artificial_cols.iter().enumerate() {
        if column >= width {
            return Err("an artificial column is out of range".into());
        }
        if options.artificial_cols[..index].contains(&column) {
            return Err("artificial columns cannot repeat".into());
        }
    }
    Ok(())
}

pub fn finish(tab: Tableau, options: FinishOptions) -> Result<Solution, String> {
    validate_tableau(&tab, &options)?;
    let start = tab.clone();
    let result = iterate(tab, &options.artificial_cols, 300, 0)?;
    let values = extract_values(&result.tableau, &options.decision_cols)?;
    let optimum = match result.status {
        Status::Optimal => extract_optimum(&result.tableau, options.report_sign)?,
        _ => None,
    };
    let names: Vec<String> = options
        .decision_cols
        .iter()
        .map(|&column| start.headers[column].name.clone())
        .collect();

    let mut summary_plain = Vec::new();
    let mut summary_latex = Vec::new();
    match result.status {
        Status::Optimal => {
            let value = optimum.unwrap_or(Rat::ZERO);
            let mut bits = Vec::new();
            let mut bits_tex = Vec::new();
            for (i, v) in values.iter().enumerate() {
                bits.push(format!("{} = {}", names[i], v.plain()));
                bits_tex.push(format!("{} = {}", var_latex(&names[i]), v.latex()));
            }
            if bits.is_empty() {
                summary_plain.push(format!(
                    "The optimal value is {} = {}.",
                    options.objective_label,
                    value.plain()
                ));
                summary_latex.push(format!(
                    "The optimal value is ${} = {}$.",
                    options.objective_label,
                    value.latex()
                ));
            } else {
                summary_plain.push(format!(
                    "The optimal value is {} = {} at {}.",
                    options.objective_label,
                    value.plain(),
                    bits.join(", ")
                ));
                summary_latex.push(format!(
                    "The optimal value is ${} = {}$ at ${}$.",
                    options.objective_label,
                    value.latex(),
                    bits_tex.join(",\\; ")
                ));
            }
            if options.report_sign < 0 {
                summary_plain.push(format!(
                    "The supplied row was treated as a maximisation Z-row; the reported minimum is C = -Z = {}.",
                    value.plain()
                ));
                summary_latex.push(format!(
                    "The supplied row was treated as a maximisation $Z$-row; the reported minimum is $C = -Z = {}$.",
                    value.latex()
                ));
            }
        }
        Status::Infeasible => {
            summary_plain.push(
                "An artificial variable is still positive, so the supplied tableau is infeasible."
                    .into(),
            );
            summary_latex.push(
                "An artificial variable is still positive, so the supplied tableau is infeasible."
                    .into(),
            );
        }
        Status::Unbounded => {
            summary_plain.push("The objective is unbounded from the supplied tableau.".into());
            summary_latex.push("The objective is unbounded from the supplied tableau.".into());
        }
    }

    let mut steps = Vec::with_capacity(result.steps.len() + 1);
    steps.push(Step {
        kind: StepKind::Start,
        phase: 0,
        tableau: start.clone(),
        pivot: None,
        entering: None,
        leaving: None,
        ratios: Vec::new(),
        row_ops: Vec::new(),
        note_plain: vec!["Continue from the tableau supplied in the editor.".into()],
        note_latex: vec!["Continue from the tableau supplied in the editor.".into()],
    });
    steps.extend(result.steps);

    Ok(Solution {
        source: Source::Tableau {
            start,
            decision_cols: options.decision_cols,
            objective_label: options.objective_label,
            report_sign: options.report_sign,
        },
        status: result.status,
        steps,
        values,
        optimum,
        iterations: result.iterations,
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
        match &sol.source {
            Source::Model { standard, .. } => {
                assert!(standard.transformed);
                assert!(!standard.artificial_cols.is_empty());
            }
            Source::Tableau { .. } => panic!("expected a model source"),
        }
    }

    #[test]
    fn solves_a_minimisation_with_two_stage_simplex() {
        let sol = solve_with_method(min_big_m_model(), Method::TwoStage).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(9)));
        assert_eq!(sol.values, vec![Rat::int(3), Rat::int(1)]);
        assert!(sol.steps.iter().any(|step| step.phase == 1));
        assert!(sol.steps.iter().any(|step| step.phase == 2));
        assert!(sol
            .summary_plain
            .iter()
            .any(|line| line.starts_with("Phase 1")));
    }

    #[test]
    fn detects_infeasibility_with_two_stage_simplex() {
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
        let sol = solve_with_method(model, Method::TwoStage).unwrap();
        assert_eq!(sol.status, Status::Infeasible);
        assert!(sol
            .summary_plain
            .iter()
            .any(|line| line.contains("Phase 1")));
    }

    #[test]
    fn two_stage_skips_phase_one_without_artificials() {
        let sol = solve_with_method(max_model(), Method::TwoStage).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(12)));
        assert!(sol.steps.iter().all(|step| step.phase == 0));
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

    fn tableau(
        headers: &[&str],
        rows: Vec<Vec<MVal>>,
        rhs: Vec<MVal>,
        basis: Vec<usize>,
        obj: Vec<MVal>,
        obj_rhs: MVal,
    ) -> Tableau {
        Tableau {
            headers: headers
                .iter()
                .map(|name| Header {
                    name: (*name).into(),
                    kind: VarKind::Decision,
                })
                .collect(),
            rows,
            rhs,
            basis,
            obj,
            obj_rhs,
        }
    }

    fn finish_options() -> FinishOptions {
        FinishOptions {
            decision_cols: vec![0, 1],
            artificial_cols: vec![],
            objective_label: "Z".into(),
            report_sign: 1,
        }
    }

    #[test]
    fn finishes_an_initial_tableau() {
        let tab = tableau(
            &["x", "y", "s1", "s2"],
            vec![
                vec![
                    Rat::ONE.into(),
                    Rat::ONE.into(),
                    Rat::ONE.into(),
                    Rat::ZERO.into(),
                ],
                vec![
                    Rat::ONE.into(),
                    Rat::int(3).into(),
                    Rat::ZERO.into(),
                    Rat::ONE.into(),
                ],
            ],
            vec![Rat::int(4).into(), Rat::int(6).into()],
            vec![2, 3],
            vec![
                Rat::int(-3).into(),
                Rat::int(-2).into(),
                Rat::ZERO.into(),
                Rat::ZERO.into(),
            ],
            Rat::ZERO.into(),
        );
        let sol = finish(tab, finish_options()).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(12)));
        assert_eq!(sol.values, vec![Rat::int(4), Rat::int(0)]);
        assert_eq!(sol.iterations, 1);
        assert_eq!(sol.steps.first().unwrap().kind, StepKind::Start);
        assert_eq!(sol.steps.last().unwrap().kind, StepKind::Optimal);
    }

    #[test]
    fn finishes_a_partially_completed_tableau() {
        let tab = tableau(
            &["x", "y", "s1", "s2"],
            vec![
                vec![
                    Rat::new(2, 3).unwrap().into(),
                    Rat::ZERO.into(),
                    Rat::ONE.into(),
                    Rat::new(-1, 3).unwrap().into(),
                ],
                vec![
                    Rat::new(1, 3).unwrap().into(),
                    Rat::ONE.into(),
                    Rat::ZERO.into(),
                    Rat::new(1, 3).unwrap().into(),
                ],
            ],
            vec![Rat::int(2).into(), Rat::int(2).into()],
            vec![2, 1],
            vec![
                Rat::int(-1).into(),
                Rat::ZERO.into(),
                Rat::ZERO.into(),
                Rat::ONE.into(),
            ],
            Rat::int(6).into(),
        );
        let sol = finish(tab, finish_options()).unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(9)));
        assert_eq!(sol.values, vec![Rat::int(3), Rat::int(1)]);
        assert_eq!(sol.iterations, 1);
        assert_eq!(sol.steps.len(), 3);
    }

    #[test]
    fn reports_a_transformed_minimum() {
        let tab = tableau(
            &["x", "y", "s1", "s2"],
            vec![
                vec![
                    Rat::new(2, 3).unwrap().into(),
                    Rat::ZERO.into(),
                    Rat::ONE.into(),
                    Rat::new(-1, 3).unwrap().into(),
                ],
                vec![
                    Rat::new(1, 3).unwrap().into(),
                    Rat::ONE.into(),
                    Rat::ZERO.into(),
                    Rat::new(1, 3).unwrap().into(),
                ],
            ],
            vec![Rat::int(2).into(), Rat::int(2).into()],
            vec![2, 1],
            vec![
                Rat::int(-1).into(),
                Rat::ZERO.into(),
                Rat::ZERO.into(),
                Rat::ONE.into(),
            ],
            Rat::int(6).into(),
        );
        let sol = finish(
            tab,
            FinishOptions {
                decision_cols: vec![0, 1],
                artificial_cols: vec![],
                objective_label: "C".into(),
                report_sign: -1,
            },
        )
        .unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(-9)));
    }

    #[test]
    fn finishes_a_tableau_with_m_terms() {
        let tab = tableau(
            &["x", "a1"],
            vec![vec![Rat::ONE.into(), Rat::ONE.into()]],
            vec![Rat::int(4).into()],
            vec![1],
            vec![
                Rat::int(-2).into(),
                MVal {
                    a: Rat::ZERO,
                    b: Rat::ONE,
                },
            ],
            Rat::ZERO.into(),
        );
        let sol = finish(
            tab,
            FinishOptions {
                decision_cols: vec![0],
                artificial_cols: vec![1],
                objective_label: "Z".into(),
                report_sign: 1,
            },
        )
        .unwrap();
        assert_eq!(sol.status, Status::Optimal);
        assert_eq!(sol.optimum, Some(Rat::int(8)));
        assert_eq!(sol.values, vec![Rat::int(4)]);
    }

    #[test]
    fn detects_infeasible_partial_tableau() {
        let tab = tableau(
            &["x", "a1"],
            vec![vec![Rat::ZERO.into(), Rat::ONE.into()]],
            vec![Rat::int(1).into()],
            vec![1],
            vec![
                Rat::ZERO.into(),
                MVal {
                    a: Rat::ZERO,
                    b: Rat::ONE,
                },
            ],
            Rat::ZERO.into(),
        );
        let sol = finish(
            tab,
            FinishOptions {
                decision_cols: vec![0],
                artificial_cols: vec![1],
                objective_label: "Z".into(),
                report_sign: 1,
            },
        )
        .unwrap();
        assert_eq!(sol.status, Status::Infeasible);
        assert!(sol.optimum.is_none());
    }

    #[test]
    fn detects_unbounded_partial_tableau() {
        let tab = tableau(
            &["x", "s1"],
            vec![vec![Rat::ZERO.into(), Rat::ONE.into()]],
            vec![Rat::int(1).into()],
            vec![1],
            vec![Rat::int(-1).into(), Rat::ZERO.into()],
            Rat::ZERO.into(),
        );
        let sol = finish(
            tab,
            FinishOptions {
                decision_cols: vec![0],
                artificial_cols: vec![],
                objective_label: "Z".into(),
                report_sign: 1,
            },
        )
        .unwrap();
        assert_eq!(sol.status, Status::Unbounded);
    }

    #[test]
    fn rejects_a_non_unit_basis_column() {
        let tab = tableau(
            &["x", "y"],
            vec![vec![Rat::int(2).into(), Rat::ONE.into()]],
            vec![Rat::int(1).into()],
            vec![0],
            vec![Rat::ZERO.into(), Rat::ZERO.into()],
            Rat::ZERO.into(),
        );
        assert!(finish(
            tab,
            FinishOptions {
                decision_cols: vec![0, 1],
                artificial_cols: vec![],
                objective_label: "Z".into(),
                report_sign: 1,
            }
        )
        .unwrap_err()
        .contains("must be 1"));
    }

    #[test]
    fn rejects_duplicate_basic_variables() {
        let tab = tableau(
            &["x", "s1"],
            vec![
                vec![Rat::ONE.into(), Rat::ZERO.into()],
                vec![Rat::ONE.into(), Rat::ONE.into()],
            ],
            vec![Rat::int(1).into(), Rat::int(2).into()],
            vec![0, 0],
            vec![Rat::ZERO.into(), Rat::ZERO.into()],
            Rat::ZERO.into(),
        );
        assert!(finish(
            tab,
            FinishOptions {
                decision_cols: vec![0, 1],
                artificial_cols: vec![],
                objective_label: "Z".into(),
                report_sign: 1,
            }
        )
        .unwrap_err()
        .contains("more than one row"));
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
