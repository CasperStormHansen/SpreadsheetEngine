use std::collections::HashMap;
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::CellAddress;
use crate::value_types::{EvaluatedValue, SingleCellValue};
use crate::formula::area_sum::AreaSum;
use crate::formula::literal::boolean_literal::BooleanLiteral;
use crate::formula::cell_reference::CellReference;
use crate::formula::conditional::Conditional;
use crate::formula::literal::number_literal::NumberLiteral;
use crate::formula::ill_formed_formula::IllFormedFormula;
use crate::formula::indirect::Indirect;
use crate::formula::literal::text_literal::TextLiteral;
use crate::formula::rand_between::RandBetween;
use crate::formula::dynamic_array::sequence::Sequence;
use crate::formula::utils::normalized_raw_formula::NormalizedRawFormula;

pub(crate) trait Formula {
    //fn evaluate(&self, spreadsheet: &Spreadsheet) -> EvaluationResult; // Todo: Delete together with implementations

    /// This method returns the child_rectangles that are required for any evaluation of the formula.
    /// Example: if the formula is of the form `IF(a,b,c)`, the initial child rectangles are those of `a`.
    //fn get_initial_child_rectangles(&self) -> HashSet<CellRectangle>; // Todo: Delete together with implementations

    /// This method returns a request for the data that is required for the first attempt at evaluation
    /// of the formula and the method that does that first attempt when supplied with that data. The data
    /// can consist of the values of all cells in specified rectangles and/or the values of sub-formulas.
    /// Example: if the formula is of the form `IF(a,b,c)`, the request will be for the value of `a`, and
    /// the evaluation method may not return a value but another request, namely for the data required to
    /// evaluate `b`.
    fn initial_data_request_and_evaluation_method(&self) -> DataRequestAndEvaluationMethod<'_>;

    /// This method returns True if the formula should be reevaluated upon any change to the spreadsheet.
    fn is_volatile(&self) -> bool;

    fn as_address(&self) -> FormulaAddress {
        self as *const Self as *const ()
    }
}

trait WellFormedFormula: Formula {
    fn try_parse(raw_formula: &NormalizedRawFormula) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! try_parse_in_order {
    ($raw_formula:expr, $($ty:ty),+ $(,)?) => {{
        let raw_formula: &str = $raw_formula;
        let raw_formula_without_whitespace = NormalizedRawFormula::from(raw_formula);
        $(
            if let Some(formula) = <$ty>::try_parse(&raw_formula_without_whitespace) {
                return Box::new(formula);
            }
        )+
        Box::new(IllFormedFormula::new(raw_formula))
    }};
}

pub(crate) fn parse(raw_formula: &str) -> Box<dyn Formula> {
    try_parse_in_order!(raw_formula,
        NumberLiteral,
        BooleanLiteral,
        TextLiteral,
        CellReference,
        AreaSum,
        Conditional,
        Indirect,
        RandBetween,
        Sequence,
    )
}

pub(crate) struct DataRequestAndEvaluationMethod<'a> {
    pub(crate) cell_rectangles: Vec<CellRectangle>,
    pub(crate) formulas: Vec<&'a dyn Formula>,
    pub(crate) evaluation_method: EvaluationMethod<'a>
}

pub(crate) type EvaluationMethod<'a> = Box<dyn Fn(EvaluationData) -> ResultOrRequest<'a> + 'a>;

pub(crate) struct EvaluationData<'a> {
    pub(crate) rectangle_to_address_value_map: &'a HashMap<CellRectangle, Vec<(CellAddress, SingleCellValue)>>,
    pub(crate) formula_to_value_map: &'a HashMap<FormulaAddress, EvaluatedValue>,
}

pub(crate) type FormulaAddress = *const ();

pub(crate) enum ResultOrRequest<'a> {
    Result(EvaluatedValue),
    Request(DataRequestAndEvaluationMethod<'a>),
}

mod cell_reference;
mod ill_formed_formula;
mod area_sum;
mod utils;
mod conditional;
mod literal;
mod indirect;
mod rand_between;
mod dynamic_array;
