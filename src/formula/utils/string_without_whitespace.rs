use std::ops::Deref;

pub struct StringWithoutWhitespace(String);

impl From<&str> for StringWithoutWhitespace {
    fn from(s: &str) -> Self {
        Self(s.chars().filter(|c| !c.is_whitespace()).collect())
    }
}

impl Deref for StringWithoutWhitespace {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}