use fmt::Display;
use fmt::Formatter;
use fmt::Result as FmtResult;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.value())
    }
}
