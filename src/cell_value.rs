#[derive(PartialEq, Debug, Clone)]
pub(crate) enum CellValue {
    Number(f64),
    Error(String),
    Unevaluated,
}
