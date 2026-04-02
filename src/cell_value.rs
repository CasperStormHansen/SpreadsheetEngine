#[derive(PartialEq, Debug, Clone)]
pub(crate) enum CellValue {
    Number(f64), // todo: To be improved. This choice means that 0.1 + 0.2 != 0.3 due to floating point precision issues.
    Error(String),
    Unevaluated,
}
