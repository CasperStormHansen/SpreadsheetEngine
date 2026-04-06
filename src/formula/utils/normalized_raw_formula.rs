use std::ops::Deref;

/// A normalized version of a raw formula string.
///
/// Normalization removes whitespace and converts the text to lowercase.
pub struct NormalizedRawFormula(String);

impl From<&str> for NormalizedRawFormula {
    fn from(s: &str) -> Self {
        Self(s.chars()
            .filter(|c| !c.is_whitespace())
            .flat_map(|c| c.to_lowercase())
            .collect())
    }
}

impl Deref for NormalizedRawFormula {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}