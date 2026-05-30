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

        for (index, char) in self.char_indices() {
            match char {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ch if ch == delimiter && depth == 0 => {
                    let delimiter_end = index + ch.len_utf8();
                    return Some((&self[..index], &self[delimiter_end..]));
                }
                _ => {}
            }
        }

        None
    }
}