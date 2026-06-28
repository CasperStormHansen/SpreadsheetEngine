use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::parse_cell_address;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{DataRequestAndEvaluationMethod, EvaluationData, Formula, ResultOrRequest, WellFormedFormula};
use crate::CellAddress;
use crate::formula::ResultOrRequest::Result;
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::value_types::SingleCellValue::Number;

pub(crate) struct CellReference {
    cell_address: CellAddress
}

impl Formula for CellReference {
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_> {
        DataRequestAndEvaluationMethod {
            cell_rectangles: vec!(CellRectangle::from_cell(self.cell_address.clone())),
            formulas: vec!(),
            evaluation_method: Box::new(|data| self.evaluate(data)),
        }
    }

    fn is_volatile(&self) -> bool {
        false
    }
}

impl CellReference {
    fn evaluate(&self, evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        let rectangle = CellRectangle::from_cell(self.cell_address.clone());
        match evaluation_data.rectangle_to_address_value_map[&rectangle].first() {
            Some((_, value)) => Result(SingleCellValue(value.clone())),
            None => Result(SingleCellValue(Number(0.0))),
        }
    }
}

impl WellFormedFormula for CellReference {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix('(')?.strip_suffix(')')?;
        let cell_address = parse_cell_address(inner)?;
        Some(Self{cell_address})
    }
}
