use std::collections::{HashMap, HashSet};
use crate::cell_lookup_structure::cell_rectangle::CellRectangle;
use crate::formula::{EvaluationData, EvaluationMethod, Formula, FormulaAddress};
use crate::{CellAddress, Spreadsheet};
use crate::formula::ResultOrRequest::{Request, Result};
use crate::value_types::{CompletedEvaluationResult, EvaluatedValue, EvaluationResult, SingleCellValue};

/// This method returns the child_rectangles that are required for any evaluation of the formula.
/// Example: if the formula is of the form `IF(a,b,c)`, the initial child rectangles are those of `a`.
pub(crate) fn get_initial_child_rectangles(formula: &dyn Formula) -> HashSet<CellRectangle> {
    let (cell_rectangles, _) = get_required_cell_rectangles_and_sub_formulas(formula);
    cell_rectangles
}

pub(crate) fn evaluate_formula(spreadsheet: &Spreadsheet, formula: &dyn Formula) -> EvaluationResult {
    let (mut cell_rectangles, mut formulas_with_methods)
        = get_required_cell_rectangles_and_sub_formulas(formula);
    let mut formulas_with_values: HashMap<FormulaAddress, EvaluatedValue> = HashMap::new();
    let mut cell_values = collect_cell_values(spreadsheet, &cell_rectangles)
        .expect("A formula should not be attempted evaluated before all its initial child rectangles have been evaluated.");
    while let Some(formula_with_method) = formulas_with_methods.pop() {
        let evaluation_data = EvaluationData {
            rectangle_to_address_value_map: &cell_values,
            formula_to_value_map: &formulas_with_values
        };
        let formula_evaluation_result = (formula_with_method.method)(evaluation_data);
        match formula_evaluation_result {
            Result(evaluated_value) => {
                formulas_with_values.insert(formula_with_method.formula.as_address(), evaluated_value);
            },
            Request(data_request_and_evaluation_method) => {
                let new_rectangles: HashSet<CellRectangle> = data_request_and_evaluation_method.cell_rectangles.into_iter().collect();
                cell_rectangles.extend(new_rectangles.iter().cloned());
                formulas_with_methods.push(FormulaWithMethod { formula: formula_with_method.formula, method: data_request_and_evaluation_method.evaluation_method });
                match collect_cell_values(spreadsheet, &new_rectangles) {
                    Some(extra_cell_values) => cell_values.extend(extra_cell_values),
                    None => return Err(cell_rectangles),
                };
                for f in data_request_and_evaluation_method.formulas {
                    let (extra_cell_rectangles, extra_formulas_with_methods)
                        = get_required_cell_rectangles_and_sub_formulas(f);
                    cell_rectangles.extend(extra_cell_rectangles.clone());
                    formulas_with_methods.extend(extra_formulas_with_methods);
                    match collect_cell_values(spreadsheet, &extra_cell_rectangles) {
                        Some(extra_cell_values) => cell_values.extend(extra_cell_values),
                        None => return Err(cell_rectangles),
                    };
                }
            }
        }
    }

    Ok(CompletedEvaluationResult(formulas_with_values[&formula.as_address()].clone(), cell_rectangles))
}

fn get_required_cell_rectangles_and_sub_formulas(formula: &dyn Formula) -> (HashSet<CellRectangle>, Vec<FormulaWithMethod<'_>>) {
    let mut cell_rectangles: HashSet<CellRectangle> = HashSet::new();
    let mut formula_with_methods: Vec<FormulaWithMethod> = Vec::new();

    let mut stack: Vec<&dyn Formula> = vec![formula];

    while let Some(f) = stack.pop() {
        let request = f.initial_data_request_and_evaluation_method();
        cell_rectangles.extend(request.cell_rectangles);
        formula_with_methods.push(FormulaWithMethod {formula: f, method: request.evaluation_method});

        stack.extend(request.formulas);
    }

    (cell_rectangles, formula_with_methods)
}

/// For each rectangle, collects the address and value of every cell that exists within it.
/// Returns `None` if any such cell has not yet been evaluated (i.e., has a `None` value).
fn collect_cell_values(spreadsheet: &Spreadsheet, rectangles: &HashSet<CellRectangle>)
    -> Option<HashMap<CellRectangle, Vec<(CellAddress, SingleCellValue)>>> {
    rectangles.iter()
        .map(|rectangle| {
            let pairs: Option<Vec<(CellAddress, SingleCellValue)>> = spreadsheet.cells
                .get_all_in_rectangle(rectangle)
                .map(|(address, cell)| cell.value.clone().map(|v| (address, v)))
                .collect();
            pairs.map(|p| (rectangle.clone(), p))
        })
        .collect()
}

struct FormulaWithMethod<'a> {
    pub(crate) formula: &'a dyn Formula,
    pub(crate) method: EvaluationMethod<'a>
}
