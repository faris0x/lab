//! Exact matrix operations with a step trace.

use mathcore::Rat;

#[derive(Clone, Debug)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<Rat>>,
    /// Number of coefficient columns before an augmented block, if any.
    pub separator: Option<usize>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<Vec<Rat>>) -> Result<Self, String> {
        if rows == 0 || cols == 0 {
            return Err("a matrix needs at least one row and one column".into());
        }
        if data.len() != rows {
            return Err("the number of rows does not match the input".into());
        }
        for row in &data {
            if row.len() != cols {
                return Err("the matrix is not rectangular".into());
            }
        }
        Ok(Matrix {
            rows,
            cols,
            data,
            separator: None,
        })
    }

    pub fn identity(n: usize) -> Self {
        let mut data = vec![vec![Rat::ZERO; n]; n];
        for (i, row) in data.iter_mut().enumerate() {
            row[i] = Rat::ONE;
        }
        Matrix {
            rows: n,
            cols: n,
            data,
            separator: None,
        }
    }

    fn swap_rows(&mut self, a: usize, b: usize) {
        self.data.swap(a, b);
    }

    fn scale_row(&mut self, r: usize, factor: Rat) -> Result<(), String> {
        for cell in &mut self.data[r] {
            *cell = cell.mul(factor)?;
        }
        Ok(())
    }

    fn add_multiple(&mut self, target: usize, source: usize, factor: Rat) -> Result<(), String> {
        if factor.is_zero() {
            return Ok(());
        }
        for j in 0..self.cols {
            let delta = self.data[source][j].mul(factor)?;
            self.data[target][j] = self.data[target][j].add(delta)?;
        }
        Ok(())
    }

    fn augment(&self, other: &Matrix) -> Result<Matrix, String> {
        if self.rows != other.rows {
            return Err("the matrices must have the same number of rows".into());
        }
        let mut data = Vec::with_capacity(self.rows);
        for i in 0..self.rows {
            let mut row = self.data[i].clone();
            row.extend(other.data[i].iter().copied());
            data.push(row);
        }
        Ok(Matrix {
            rows: self.rows,
            cols: self.cols + other.cols,
            data,
            separator: Some(self.cols),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Step {
    pub kind: &'static str,
    pub matrix: Matrix,
    pub description: String,
    pub description_latex: String,
    pub pivot: Option<(usize, usize)>,
}

impl Step {
    fn new(
        kind: &'static str,
        matrix: Matrix,
        description: String,
        description_latex: String,
        pivot: Option<(usize, usize)>,
    ) -> Self {
        Step {
            kind,
            matrix,
            description,
            description_latex,
            pivot,
        }
    }
}

pub struct Rref {
    pub matrix: Matrix,
    pub steps: Vec<Step>,
    pub pivots: Vec<usize>,
}

fn rlatex(r: usize) -> String {
    format!("R_{{{}}}", r + 1)
}

fn row_op_plain(target: usize, source: usize, factor: Rat) -> String {
    let t = target + 1;
    let s = source + 1;
    if factor == Rat::ONE {
        format!("R{t} ← R{t} − R{s}")
    } else if factor == Rat::int(-1) {
        format!("R{t} ← R{t} + R{s}")
    } else if factor.is_negative() {
        format!("R{t} ← R{t} + {}R{s}", factor.abs().plain())
    } else {
        format!("R{t} ← R{t} − {}R{s}", factor.plain())
    }
}

fn row_op_latex(target: usize, source: usize, factor: Rat) -> String {
    let t = target + 1;
    let s = source + 1;
    if factor == Rat::ONE {
        format!("R_{{{t}}} \\leftarrow R_{{{t}}} - R_{{{s}}}")
    } else if factor == Rat::int(-1) {
        format!("R_{{{t}}} \\leftarrow R_{{{t}}} + R_{{{s}}}")
    } else if factor.is_negative() {
        format!(
            "R_{{{t}}} \\leftarrow R_{{{t}}} + {}R_{{{s}}}",
            factor.abs().latex()
        )
    } else {
        format!(
            "R_{{{t}}} \\leftarrow R_{{{t}}} - {}R_{{{s}}}",
            factor.latex()
        )
    }
}

fn elimination_words(target: usize, source: usize, factor: Rat) -> String {
    let t = target + 1;
    let s = source + 1;
    if factor == Rat::ONE {
        format!("Subtract row {s} from row {t}")
    } else if factor == Rat::int(-1) {
        format!("Add row {s} to row {t}")
    } else if factor.is_negative() {
        format!("Add {} times row {s} to row {t}", factor.abs().plain())
    } else {
        format!("Subtract {} times row {s} from row {t}", factor.plain())
    }
}

fn elimination_words_latex(target: usize, source: usize, factor: Rat) -> String {
    let t = rlatex(target);
    let s = rlatex(source);
    if factor == Rat::ONE {
        format!("Subtract ${s}$ from ${t}$")
    } else if factor == Rat::int(-1) {
        format!("Add ${s}$ to ${t}$")
    } else if factor.is_negative() {
        format!("Add ${}$ times ${s}$ to ${t}$", factor.abs().latex())
    } else {
        format!("Subtract ${}$ times ${s}$ from ${t}$", factor.latex())
    }
}

pub fn rref(mut m: Matrix, start: &str, start_latex: &str) -> Result<Rref, String> {
    let mut steps = vec![Step::new(
        "start",
        m.clone(),
        start.to_string(),
        start_latex.to_string(),
        None,
    )];
    let mut pivots = Vec::new();
    let mut pivot_row = 0usize;

    for col in 0..m.cols {
        if pivot_row >= m.rows {
            break;
        }
        let mut found = None;
        for (r, row) in m.data.iter().enumerate().take(m.rows).skip(pivot_row) {
            if !row[col].is_zero() {
                found = Some(r);
                break;
            }
        }
        let Some(r) = found else {
            continue;
        };

        if r != pivot_row {
            m.swap_rows(r, pivot_row);
            steps.push(Step::new(
                "swap",
                m.clone(),
                format!(
                    "The first nonzero entry in column {} is in row {}. Swap rows {} and {} so the pivot sits in row {}.",
                    col + 1,
                    r + 1,
                    r + 1,
                    pivot_row + 1,
                    pivot_row + 1
                ),
                format!(
                    "The first nonzero entry in column ${}$ is in row ${}$. Swap rows ${}$ and ${}$ so the pivot sits in row ${}$.",
                    col + 1,
                    r + 1,
                    rlatex(r),
                    rlatex(pivot_row),
                    pivot_row + 1
                ),
                Some((pivot_row, col)),
            ));
        }

        let pivot = m.data[pivot_row][col];
        if pivot != Rat::ONE {
            let factor = Rat::ONE.div(pivot)?;
            m.scale_row(pivot_row, factor)?;
            steps.push(Step::new(
                "scale",
                m.clone(),
                format!(
                    "Divide row {} by {} so the pivot becomes 1.",
                    pivot_row + 1,
                    pivot.plain()
                ),
                format!(
                    "Divide row ${}$ by ${}$ so the pivot becomes $1$.",
                    rlatex(pivot_row),
                    pivot.latex()
                ),
                Some((pivot_row, col)),
            ));
        }

        for r in 0..m.rows {
            if r == pivot_row {
                continue;
            }
            let factor = m.data[r][col];
            if factor.is_zero() {
                continue;
            }
            m.add_multiple(r, pivot_row, factor.neg())?;
            steps.push(Step::new(
                "eliminate",
                m.clone(),
                format!(
                    "{} ({}). This clears the entry in column {}.",
                    elimination_words(r, pivot_row, factor),
                    row_op_plain(r, pivot_row, factor),
                    col + 1
                ),
                format!(
                    "{} (${}$). This clears the entry in column ${}$.",
                    elimination_words_latex(r, pivot_row, factor),
                    row_op_latex(r, pivot_row, factor),
                    col + 1
                ),
                Some((pivot_row, col)),
            ));
        }

        pivots.push(col);
        pivot_row += 1;
    }

    Ok(Rref {
        matrix: m,
        steps,
        pivots,
    })
}

pub struct Determinant {
    pub value: Rat,
    pub steps: Vec<Step>,
}

pub fn determinant(m: Matrix) -> Result<Determinant, String> {
    if m.rows != m.cols {
        return Err("the determinant needs a square matrix".into());
    }
    let n = m.rows;
    let mut m = m;
    let mut value = Rat::ONE;
    let mut steps = vec![Step::new(
        "start",
        m.clone(),
        "Start with A. Clear below each pivot in turn; row swaps flip the sign, and the determinant is the product of the pivots.".to_string(),
        "Start with $A$. Clear below each pivot in turn; row swaps flip the sign, and the determinant is the product of the pivots.".to_string(),
        None,
    )];

    for col in 0..n {
        let mut found = None;
        for (r, row) in m.data.iter().enumerate().take(n).skip(col) {
            if !row[col].is_zero() {
                found = Some(r);
                break;
            }
        }
        let Some(r) = found else {
            value = Rat::ZERO;
            steps.push(Step::new(
                "zero",
                m.clone(),
                format!("No nonzero pivot in column {}, so det(A) = 0.", col + 1),
                format!(
                    "No nonzero pivot in column ${}$, so $\\det(A) = 0$.",
                    col + 1
                ),
                None,
            ));
            break;
        };

        if r != col {
            m.swap_rows(r, col);
            value = value.neg();
            steps.push(Step::new(
                "swap",
                m.clone(),
                format!(
                    "Bring a nonzero entry into the pivot position: swap rows {} and {}. A swap changes the sign of the determinant.",
                    col + 1,
                    r + 1
                ),
                format!(
                    "Bring a nonzero entry into the pivot position: swap ${}$ and ${}$. A swap changes the sign of the determinant.",
                    rlatex(col),
                    rlatex(r)
                ),
                Some((col, col)),
            ));
        }

        let pivot = m.data[col][col];
        value = value.mul(pivot)?;
        steps.push(Step::new(
            "pivot",
            m.clone(),
            format!(
                "The pivot is {}. Multiply the running determinant by it.",
                pivot.plain()
            ),
            format!(
                "The pivot is ${}$. Multiply the running determinant by it.",
                pivot.latex()
            ),
            Some((col, col)),
        ));

        for r in (col + 1)..n {
            let factor = m.data[r][col].div(pivot)?;
            if factor.is_zero() {
                continue;
            }
            m.add_multiple(r, col, factor.neg())?;
            steps.push(Step::new(
                "eliminate",
                m.clone(),
                format!(
                    "{} ({}). This clears the entry below the pivot; row replacement leaves the determinant unchanged.",
                    elimination_words(r, col, factor),
                    row_op_plain(r, col, factor)
                ),
                format!(
                    "{} (${}$). This clears the entry below the pivot; row replacement leaves the determinant unchanged.",
                    elimination_words_latex(r, col, factor),
                    row_op_latex(r, col, factor)
                ),
                Some((col, col)),
            ));
        }
    }

    Ok(Determinant { value, steps })
}

pub fn inverse(m: Matrix) -> Result<(Option<Matrix>, Rref), String> {
    if m.rows != m.cols {
        return Err("the inverse needs a square matrix".into());
    }
    let n = m.rows;
    let augmented = m.augment(&Matrix::identity(n))?;
    let reduced = rref(
        augmented,
        "Start with the augmented matrix [A | I]. Row operations on the left block also change the right block; when the left block becomes I, the right block is A^{-1}.",
        "Start with the augmented matrix $[A \\mid I]$. Row operations on the left block also change the right block; when the left block becomes $I$, the right block is $A^{-1}$.",
    )?;
    let rank_a = reduced.pivots.iter().filter(|&&c| c < n).count();
    if rank_a < n {
        return Ok((None, reduced));
    }
    let mut data = vec![vec![Rat::ZERO; n]; n];
    for (i, row) in data.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = reduced.matrix.data[i][n + j];
        }
    }
    Ok((
        Some(Matrix {
            rows: n,
            cols: n,
            data,
            separator: None,
        }),
        reduced,
    ))
}

pub struct Nullspace {
    pub basis: Vec<Vec<Rat>>,
    pub rref: Rref,
}

pub fn nullspace(m: Matrix) -> Result<Nullspace, String> {
    let reduced = rref(
        m,
        "Start with A. Row-reduce it; each free column gives one nullspace basis vector.",
        "Start with $A$. Row-reduce it; each free column gives one nullspace basis vector.",
    )?;
    let cols = reduced.matrix.cols;
    let mut basis = Vec::new();
    for free in 0..cols {
        if reduced.pivots.contains(&free) {
            continue;
        }
        let mut vector = vec![Rat::ZERO; cols];
        vector[free] = Rat::ONE;
        for (i, &pivot_col) in reduced.pivots.iter().enumerate() {
            vector[pivot_col] = reduced.matrix.data[i][free].neg();
        }
        basis.push(vector);
    }
    Ok(Nullspace {
        basis,
        rref: reduced,
    })
}

pub enum SolveKind {
    Unique(Vec<Rat>),
    Infinite {
        particular: Vec<Rat>,
        nullspace: Vec<Vec<Rat>>,
    },
    Inconsistent,
}

pub struct SolvedSystem {
    pub kind: SolveKind,
    pub rref: Rref,
}

pub fn solve(m: Matrix, rhs: &[Rat]) -> Result<SolvedSystem, String> {
    if rhs.len() != m.rows {
        return Err("the right-hand side needs one entry per row".into());
    }
    let rhs_matrix = Matrix {
        rows: m.rows,
        cols: 1,
        data: rhs.iter().map(|v| vec![*v]).collect(),
        separator: None,
    };
    let augmented = m.augment(&rhs_matrix)?;
    let reduced = rref(
        augmented,
        "Start with the augmented matrix [A | b]. Row-reduce the whole augmented matrix; the reduced system has the same solutions as the original.",
        "Start with the augmented matrix $[A \\mid \\mathbf{b}]$. Row-reduce the whole augmented matrix; the reduced system has the same solutions as the original.",
    )?;
    let cols = m.cols;

    for row in &reduced.matrix.data {
        let all_zero = row[..cols].iter().all(|v| v.is_zero());
        if all_zero && !row[cols].is_zero() {
            return Ok(SolvedSystem {
                kind: SolveKind::Inconsistent,
                rref: reduced,
            });
        }
    }

    if reduced.pivots.len() == cols {
        let mut values = vec![Rat::ZERO; cols];
        for (i, &pivot_col) in reduced.pivots.iter().enumerate() {
            if pivot_col < cols {
                values[pivot_col] = reduced.matrix.data[i][cols];
            }
        }
        return Ok(SolvedSystem {
            kind: SolveKind::Unique(values),
            rref: reduced,
        });
    }

    let mut particular = vec![Rat::ZERO; cols];
    for (i, &pivot_col) in reduced.pivots.iter().enumerate() {
        if pivot_col < cols {
            particular[pivot_col] = reduced.matrix.data[i][cols];
        }
    }

    let mut null_basis = Vec::new();
    for free in 0..cols {
        if reduced.pivots.contains(&free) {
            continue;
        }
        let mut vector = vec![Rat::ZERO; cols];
        vector[free] = Rat::ONE;
        for (i, &pivot_col) in reduced.pivots.iter().enumerate() {
            vector[pivot_col] = reduced.matrix.data[i][free].neg();
        }
        null_basis.push(vector);
    }

    Ok(SolvedSystem {
        kind: SolveKind::Infinite {
            particular,
            nullspace: null_basis,
        },
        rref: reduced,
    })
}

pub fn matrix_latex(m: &Matrix, pivot: Option<(usize, usize)>) -> String {
    let (open, close) = match m.separator {
        Some(sep) => {
            let mut spec = String::new();
            for j in 0..m.cols {
                if j == sep {
                    spec.push('|');
                }
                spec.push('c');
            }
            (
                format!("\\left[\\begin{{array}}{{{spec}}}\n"),
                "\\end{array}\\right]".to_string(),
            )
        }
        None => (
            "\\begin{bmatrix}\n".to_string(),
            "\\end{bmatrix}".to_string(),
        ),
    };
    let mut out = open;
    for i in 0..m.rows {
        for (j, cell) in m.data[i].iter().enumerate().take(m.cols) {
            if j > 0 {
                out.push_str(" & ");
            }
            let value = cell.latex();
            if pivot == Some((i, j)) {
                out.push_str("\\boxed{");
                out.push_str(&value);
                out.push('}');
            } else {
                out.push_str(&value);
            }
        }
        if i + 1 < m.rows {
            out.push_str(" \\\\\n");
        }
    }
    out.push('\n');
    out.push_str(&close);
    out
}

pub fn vector_latex(v: &[Rat]) -> String {
    let mut out = String::from("\\begin{bmatrix}\n");
    for (i, value) in v.iter().enumerate() {
        out.push_str(&value.latex());
        if i + 1 < v.len() {
            out.push_str(" \\\\\n");
        }
    }
    out.push_str("\n\\end{bmatrix}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mat(rows: &[&[i64]]) -> Matrix {
        let data: Vec<Vec<Rat>> = rows
            .iter()
            .map(|row| row.iter().map(|v| Rat::int(*v)).collect())
            .collect();
        Matrix::new(rows.len(), rows[0].len(), data).unwrap()
    }

    fn values(v: &[Rat]) -> Vec<(i64, i64)> {
        v.iter().map(|r| (r.n, r.d)).collect()
    }

    #[test]
    fn rref_reaches_the_identity() {
        let reduced = rref(mat(&[&[1, 2], &[3, 4]]), "Start with A.", "Start with $A$.").unwrap();
        assert_eq!(reduced.pivots, vec![0, 1]);
        assert_eq!(reduced.matrix.data[0][0], Rat::ONE);
        assert_eq!(reduced.matrix.data[1][1], Rat::ONE);
        assert_eq!(reduced.matrix.data[0][1], Rat::ZERO);
        assert_eq!(reduced.matrix.data[1][0], Rat::ZERO);
    }

    #[test]
    fn determinant_handles_swaps_and_fractions() {
        let det = determinant(mat(&[&[1, 2], &[3, 4]])).unwrap();
        assert_eq!(det.value, Rat::int(-2));
        let singular = determinant(mat(&[&[1, 2], &[2, 4]])).unwrap();
        assert_eq!(singular.value, Rat::ZERO);
    }

    #[test]
    fn inverse_of_a_two_by_two() {
        let (inv, _) = inverse(mat(&[&[1, 2], &[3, 4]])).unwrap();
        let inv = inv.unwrap();
        assert_eq!(inv.data[0][0], Rat::int(-2));
        assert_eq!(inv.data[0][1], Rat::ONE);
        assert_eq!(inv.data[1][0], Rat::new(3, 2).unwrap());
        assert_eq!(inv.data[1][1], Rat::new(-1, 2).unwrap());
        assert!(inverse(mat(&[&[1, 2], &[2, 4]])).unwrap().0.is_none());
    }

    #[test]
    fn nullspace_basis_for_a_dependent_matrix() {
        let space = nullspace(mat(&[&[1, 2], &[2, 4]])).unwrap();
        assert_eq!(space.basis.len(), 1);
        assert_eq!(values(&space.basis[0]), vec![(-2, 1), (1, 1)]);
        let full = nullspace(mat(&[&[1, 0], &[0, 1]])).unwrap();
        assert!(full.basis.is_empty());
    }

    #[test]
    fn augmented_matrices_keep_a_division_bar() {
        let a = mat(&[&[1, 2], &[3, 4]]);
        let augmented = a.augment(&Matrix::identity(2)).unwrap();
        assert_eq!(augmented.separator, Some(2));
        let latex = matrix_latex(&augmented, None);
        assert!(latex.contains("cc|cc"), "{latex}");
        assert!(latex.contains("\\left["), "{latex}");
    }

    #[test]
    fn explanations_say_what_the_step_does() {
        let reduced = rref(mat(&[&[2, 1], &[1, 3]]), "Start with A.", "Start with $A$.").unwrap();
        let text = reduced
            .steps
            .iter()
            .map(|step| step.description.clone())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(text.contains("pivot"), "{text}");
        assert!(text.contains("clears the entry"), "{text}");
    }

    #[test]
    fn solves_unique_infinite_and_inconsistent_systems() {
        let unique = solve(mat(&[&[1, 2], &[3, 4]]), &[Rat::int(5), Rat::int(6)]).unwrap();
        match unique.kind {
            SolveKind::Unique(values) => {
                assert_eq!(values, vec![Rat::int(-4), Rat::new(9, 2).unwrap()]);
            }
            _ => panic!("expected a unique solution"),
        }

        let infinite = solve(mat(&[&[1, 1]]), &[Rat::int(2)]).unwrap();
        match infinite.kind {
            SolveKind::Infinite {
                particular,
                nullspace,
            } => {
                assert_eq!(particular, vec![Rat::int(2), Rat::ZERO]);
                assert_eq!(values(&nullspace[0]), vec![(-1, 1), (1, 1)]);
            }
            _ => panic!("expected infinitely many solutions"),
        }

        let inconsistent = solve(mat(&[&[1, 1], &[1, 1]]), &[Rat::int(1), Rat::int(2)]).unwrap();
        assert!(matches!(inconsistent.kind, SolveKind::Inconsistent));
    }
}
