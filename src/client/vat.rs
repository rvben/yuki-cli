use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::YukiError;

use super::local_name;
use super::soap_client::{SoapClient, SoapEnvelope};

const BASE_URL: &str = "https://api.yukiworks.nl/ws/Vat.asmx";

/// A VAT return record for a specific period.
#[derive(Debug, Clone)]
pub struct VatReturn {
    pub period: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
}

/// An active VAT code definition.
#[derive(Debug, Clone)]
pub struct VatCode {
    pub code: String,
    pub description: String,
}

/// Client for the Yuki VAT SOAP service.
pub struct VatClient {
    soap: SoapClient,
}

impl VatClient {
    pub fn new() -> Self {
        Self {
            soap: SoapClient::new(BASE_URL),
        }
    }

    fn require_session(&self) -> Result<&str, YukiError> {
        self.soap.session_id().ok_or_else(|| {
            YukiError::AuthFailed("not authenticated — call authenticate() first".to_string())
        })
    }

    /// Authenticate with the Yuki API and store the session ID.
    pub async fn authenticate(&mut self, api_key: &str) -> Result<String, YukiError> {
        self.soap.authenticate(api_key).await
    }

    /// Retrieve the list of VAT returns.
    pub async fn vat_return_list(&self) -> Result<Vec<VatReturn>, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("VatReturnList").session(session).build();
        let body = self.soap.call("VatReturnList", envelope).await?;
        Self::parse_vat_returns(&body)
    }

    /// Retrieve all active VAT codes.
    pub async fn active_vat_codes(&self) -> Result<Vec<VatCode>, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("ActiveVatCodes").session(session).build();
        let body = self.soap.call("ActiveVatCodes", envelope).await?;
        Self::parse_vat_codes(&body)
    }

    /// Parse a VatReturnList SOAP response into a list of `VatReturn` values.
    pub fn parse_vat_returns(xml: &str) -> Result<Vec<VatReturn>, YukiError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut returns = Vec::new();
        let mut in_item = false;
        let mut field: Option<String> = None;
        let mut current = VatReturn {
            period: String::new(),
            status: String::new(),
            start_date: String::new(),
            end_date: String::new(),
        };
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = local_name(e.name().as_ref()).to_string();
                    match local.as_str() {
                        "VatReturn" => {
                            in_item = true;
                            current = VatReturn {
                                period: String::new(),
                                status: String::new(),
                                start_date: String::new(),
                                end_date: String::new(),
                            };
                        }
                        "Period" | "Status" | "StartDate" | "EndDate" if in_item => {
                            field = Some(local);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref f) = field {
                        let text = e
                            .unescape()
                            .map_err(|e| YukiError::Xml(e.to_string()))?
                            .trim()
                            .to_string();
                        match f.as_str() {
                            "Period" => current.period = text,
                            "Status" => current.status = text,
                            "StartDate" => current.start_date = text,
                            "EndDate" => current.end_date = text,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = local_name(e.name().as_ref()).to_string();
                    match local.as_str() {
                        "Period" | "Status" | "StartDate" | "EndDate" => field = None,
                        "VatReturn" if in_item => {
                            returns.push(current.clone());
                            in_item = false;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(YukiError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(returns)
    }

    /// Parse an ActiveVatCodes SOAP response into a list of `VatCode` values.
    pub fn parse_vat_codes(xml: &str) -> Result<Vec<VatCode>, YukiError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut codes = Vec::new();
        let mut in_item = false;
        let mut field: Option<String> = None;
        let mut current = VatCode {
            code: String::new(),
            description: String::new(),
        };
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = local_name(e.name().as_ref()).to_string();
                    match local.as_str() {
                        "VatCode" => {
                            in_item = true;
                            current = VatCode {
                                code: String::new(),
                                description: String::new(),
                            };
                        }
                        "Code" | "Description" if in_item => {
                            field = Some(local);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref f) = field {
                        let text = e
                            .unescape()
                            .map_err(|e| YukiError::Xml(e.to_string()))?
                            .trim()
                            .to_string();
                        match f.as_str() {
                            "Code" => current.code = text,
                            "Description" => current.description = text,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = local_name(e.name().as_ref()).to_string();
                    match local.as_str() {
                        "Code" | "Description" => field = None,
                        "VatCode" if in_item => {
                            codes.push(current.clone());
                            in_item = false;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(YukiError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(codes)
    }
}

impl Default for VatClient {
    fn default() -> Self {
        Self::new()
    }
}
