pub mod flight;
pub mod ldjsonitem;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnkerdeRawDocument {
    pub flight: flight::FlightPayload,

    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonScript>,
}
