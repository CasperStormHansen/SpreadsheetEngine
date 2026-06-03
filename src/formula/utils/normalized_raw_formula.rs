use std::ops::Deref;

/// A normalized version of a raw formula string.
///
/// Normalization removes whitespace and converts the text to lowercase.
pub struct NormalizedRawFormula(String);

impl From<&str> for NormalizedRawFormula {
    fn from(s: &str) -> Self {
        let mut result = String::new();
        let mut in_string = false;
        for c in s.chars() {
            if c == '"' {
                in_string = !in_string;
                result.push(c);
            } else if in_string {
                result.push(c);
            } else if !c.is_whitespace() {
                result.extend(c.to_lowercase());
            }
        }
        Self(result)
    }
}

impl Deref for NormalizedRawFormula {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}