use crate::formula::{DataRequestAndEvaluationMethod, EvaluationData, Formula, ResultOrRequest};
use crate::formula::ResultOrRequest::Result;
use crate::value_types::EvaluatedValue::SingleCellValue;
use crate::value_types::SingleCellValue::Error;

pub(crate) struct IllFormedFormula {
    error_message: String
}

impl Formula for IllFormedFormula {
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_> {
        DataRequestAndEvaluationMethod::empty_request(|data| self.evaluate(data))
    }

    fn is_volatile(&self) -> bool {
        false
    }
}

impl IllFormedFormula {
    pub(crate) fn new(error_message: &str) -> Self {
        Self { error_message: error_message.to_string() } // Todo: needs improvement
    }

    fn evaluate(&self, _evaluation_data: EvaluationData) -> ResultOrRequest<'_> {
        Result(SingleCellValue(Error(self.error_message.clone())))
    }
}
