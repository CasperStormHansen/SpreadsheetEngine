use crate::CellAddress;

pub(crate) fn parse_cell_address(text: &str) -> Option<CellAddress> {
    let (column, row) = text.split_once(',')?;
    Some(CellAddress::new(column.parse().ok()?, row.parse().ok()?))
}

pub(crate) trait SplitOnceOutsideParentheses {
    fn split_once_outside_parentheses(&self, delimiter: char) -> Option<(&str, &str)>;
}

impl SplitOnceOutsideParentheses for str {
    fn split_once_outside_parentheses(&self, delimiter: char) -> Option<(&str, &str)> {
        let mut depth = 0usize;
        let mut in_string = false;

        for (index, ch) in self.char_indices() {
            if ch == '"' {
                in_string = !in_string;
            } else if in_string {
                // skip content inside quoted strings
            } else if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth = depth.saturating_sub(1);
            } else if ch == delimiter && depth == 0 {
                let delimiter_end = index + ch.len_utf8();
                return Some((&self[..index], &self[delimiter_end..]));
            }
        }

        None
    }
}