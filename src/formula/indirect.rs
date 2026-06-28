use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::utils::common_parsing::parse_cell_address;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;
use crate::formula::{parse, DataRequestAndEvaluationMethod, EvaluationData, Formula, ResultOrRequest, WellFormedFormula};
use crate::formula::ResultOrRequest::{Request, Result};
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::value_types::SingleCellValue::{Error, Number, Text};

pub(crate) struct Indirect {
    reference: Box<dyn Formula>,
}

impl Formula for Indirect {
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_> {
        DataRequestAndEvaluationMethod::with_formulas(
            vec![self.reference.as_ref()],
            |data| self.evaluate_initial(data),
        )
    }

    fn is_volatile(&self) -> bool {
        self.reference.is_volatile()
    }
}

impl Indirect {
    fn evaluate_initial(&self, evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        match &evaluation_data.formula_to_value_map[&self.reference.as_address()] {
            SingleCellValue(Text(text)) => {
                if let Some(cell_address) = parse_cell_address(text) {
                    let rectangle = CellRectangle::from_cell(cell_address);
                    Request(DataRequestAndEvaluationMethod::with_rectangles(
                        vec![rectangle.clone()],
                        move |data| self.evaluate_resolved(data, &rectangle),
                    ))
                } else {
                    Result(SingleCellValue(Error("Indirect reference is not a valid cell address".to_string())))
                }
            }
            _ => Result(SingleCellValue(Error("Indirect reference is not text".to_string()))),
        }
    }

    fn evaluate_resolved(&self, evaluation_data: EvaluationData, rectangle: &CellRectangle) -> ResultOrRequest<'_> {
        match evaluation_data.rectangle_to_address_value_map[rectangle].first() {
            Some((_, value)) => Result(SingleCellValue(value.clone())),
            None => Result(SingleCellValue(Number(0.0))),
        }
    }
}

impl WellFormedFormula for Indirect {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self> {
        let inner = raw_formula.strip_prefix("indirect(")?.strip_suffix(')')?;
        let reference = parse(inner);
        Some(Self{ reference })
    }
}
