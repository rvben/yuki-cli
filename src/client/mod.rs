pub mod accounting;
pub mod soap_client;

pub use soap_client::{SoapClient, SoapEnvelope};

/// Strip any XML namespace prefix, returning only the local name.
pub(crate) fn local_name(name: &[u8]) -> &str {
    let s = std::str::from_utf8(name).unwrap_or("");
    s.rfind(':').map(|i| &s[i + 1..]).unwrap_or(s)
}
