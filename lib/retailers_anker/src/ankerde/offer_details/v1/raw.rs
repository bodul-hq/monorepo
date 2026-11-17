use flight::FlightPayload;
use ldjsonitem::LdJsonScript;
use serde::Deserialize;
pub mod flight;
pub mod ldjsonitem;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnkerdeRawDocument {
    pub flight: FlightPayload,

    #[serde(rename = "ld-json")]
    pub ld_json: Vec<LdJsonScript>,
}
