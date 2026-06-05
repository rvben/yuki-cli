//! Typed async client for the Yuki bookkeeping SOAP API.
//!
//! The transport ([`client::soap_client::SoapClient`]) accepts a caller-provided
//! [`reqwest::Client`] via [`client::soap_client::SoapClient::with_client`], so a
//! long-running consumer can share a single pooled client; [`new`] is a
//! convenience that builds its own default client per instance.
//!
//! [`new`]: client::soap_client::SoapClient::new

pub mod client;
pub mod error;
pub mod period;
