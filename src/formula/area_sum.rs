use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::parse_cell_address;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{DataRequestAndEvaluationMethod, EvaluationData, Formula, ResultOrRequest, WellFormedFormula};
use crate::value_types::SingleCellValue::{Boolean, Error, Number, Text};
use crate::formula::ResultOrRequest::Result;
use crate::value_types::EvaluatedValue::SingleCellValue;

pub(crate) struct AreaSum {
    area: CellRectangle
}

impl Formula for AreaSum {
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_> {
        DataRequestAndEvaluationMethod::with_rectangles(
            vec![self.area.clone()],
            |data| self.evaluate(data),
        )
    }

    fn is_volatile(&self) -> bool {
        false
    }
}

impl AreaSum {
    fn evaluate(&self, evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        let mut sum = 0.0;
        for (_, value) in &evaluation_data.rectangle_to_address_value_map[&self.area] {
            match value {
                Number(number) =>
                    sum += number, // todo: overflow/underflow handling
                Boolean(_) =>
                    return Result(SingleCellValue(Error("Summing over area with boolean".to_string()))),
                Text(_) =>
                    return Result(SingleCellValue(Error("Summing over area with text".to_string()))),
                Error(_) =>
                    return Result(SingleCellValue(Error("Summing over area with error".to_string()))), // todo: consider refactoring so this does not have to be handled here
            }
        }

        Result(SingleCellValue(Number(sum)))
    }
}

impl WellFormedFormula for AreaSum {
    // TODO: Accept cell addresses in the letter-number format.
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("sum(")?.strip_suffix(')')?;
        let (left, right) = inner.split_once(':')?;
        let upper_left = parse_cell_address(left)?;
        let lower_right = parse_cell_address(right)?;
        let rectangle = CellRectangle::new(upper_left, lower_right)?;
        Some(Self{area: rectangle})
    }
}
