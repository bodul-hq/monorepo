use fmt::Display;
use fmt::Formatter;
use fmt::Result;

use std::fmt;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RetailerOfferId(pub String);

impl RetailerOfferId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl Display for RetailerOfferId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.value())
    }
}
