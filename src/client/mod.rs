use quick_xml::Reader;
use quick_xml::events::Event;
use reqwest::Client;

use crate::error::YukiError;

const YUKI_NS: &str = "http://www.theyukicompany.com/";
const SOAP_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";

/// Builder for SOAP XML request envelopes.
pub struct SoapEnvelope {
    operation: String,
    session_id: Option<String>,
    params: Vec<(String, String)>,
}

impl SoapEnvelope {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            session_id: None,
            params: Vec::new(),
        }
    }

    pub fn session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn param(mut self, name: &str, value: &str) -> Self {
        self.params.push((name.to_string(), value.to_string()));
        self
    }

    pub fn build(self) -> String {
        let mut body = String::new();

        if let Some(sid) = &self.session_id {
            body.push_str(&format!("      <yuki:sessionID>{sid}</yuki:sessionID>\n"));
        }

        for (name, value) in &self.params {
            body.push_str(&format!("      <yuki:{name}>{value}</yuki:{name}>\n"));
        }

        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="{SOAP_NS}"
               xmlns:yuki="{YUKI_NS}">
  <soap:Body>
    <yuki:{op}>
{body}    </yuki:{op}>
  </soap:Body>
</soap:Envelope>"#,
            op = self.operation,
        )
    }
}

/// HTTP transport client for the Yuki SOAP API.
pub struct SoapClient {
    http: Client,
    base_url: String,
    session_id: Option<String>,
}

impl SoapClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.to_string(),
            session_id: None,
        }
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Build the SOAPAction header value for a given operation.
    pub fn soap_action(_service: &str, operation: &str) -> String {
        format!("{YUKI_NS}{operation}")
    }

    /// POST a SOAP envelope and return the raw response body.
    pub async fn call(&self, operation: &str, envelope: String) -> Result<String, YukiError> {
        let action = Self::soap_action("", operation);
        let response = self
            .http
            .post(&self.base_url)
            .header("Content-Type", "text/xml; charset=utf-8")
            .header("SOAPAction", format!("\"{action}\""))
            .body(envelope)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status == 401 || status == 403 {
            return Err(YukiError::AuthFailed(format!("HTTP {status}")));
        }
        if status == 429 {
            return Err(YukiError::RateLimited);
        }
        if !status.is_success() {
            return Err(YukiError::Http {
                status: status.as_u16(),
                body,
            });
        }

        Ok(body)
    }

    /// Authenticate with an API key, storing the returned session ID.
    pub async fn authenticate(&mut self, api_key: &str) -> Result<String, YukiError> {
        let envelope = SoapEnvelope::new("Authenticate")
            .param("accessKey", api_key)
            .build();

        let body = self.call("Authenticate", envelope).await?;
        let session = Self::parse_single_result(&body, "AuthenticateResult")?;
        self.session_id = Some(session.clone());
        Ok(session)
    }

    /// Extract the text content of a single named element from a SOAP response.
    ///
    /// Returns an error if a SOAP fault is found or the element is missing.
    pub fn parse_single_result(xml: &str, result_tag: &str) -> Result<String, YukiError> {
        // Check for fault first.
        if let Some(err) = Self::parse_soap_fault(xml) {
            return Err(err);
        }

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut inside_target = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if local == result_tag {
                        inside_target = true;
                    }
                }
                Ok(Event::Text(ref e)) if inside_target => {
                    let text = e
                        .unescape()
                        .map_err(|e| YukiError::Xml(e.to_string()))?
                        .trim()
                        .to_string();
                    if !text.is_empty() {
                        return Ok(text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if local == result_tag {
                        inside_target = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(YukiError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Err(YukiError::Xml(format!(
            "element '{result_tag}' not found in response"
        )))
    }

    /// Extract the inner XML content of the SOAP Body element.
    pub fn parse_xml_body(xml: &str) -> Result<String, YukiError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut inside_body = false;
        let mut depth: u32 = 0;
        let mut body_content = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if !inside_body && local == "Body" {
                        inside_body = true;
                        depth = 0;
                    } else if inside_body {
                        depth += 1;
                        let tag = std::str::from_utf8(name.as_ref()).unwrap_or("");
                        body_content.push('<');
                        body_content.push_str(tag);
                        body_content.push('>');
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if inside_body && local == "Body" && depth == 0 {
                        break;
                    } else if inside_body {
                        depth = depth.saturating_sub(1);
                        let tag = std::str::from_utf8(name.as_ref()).unwrap_or("");
                        body_content.push_str("</");
                        body_content.push_str(tag);
                        body_content.push('>');
                    }
                }
                Ok(Event::Text(ref e)) if inside_body => {
                    let text = e.unescape().map_err(|e| YukiError::Xml(e.to_string()))?;
                    body_content.push_str(&text);
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(YukiError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(body_content)
    }

    /// Detect a SOAP fault in the response and return it as a `YukiError`.
    ///
    /// Returns `None` if no fault is present.
    pub fn parse_soap_fault(xml: &str) -> Option<YukiError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut in_fault = false;
        let mut in_faultcode = false;
        let mut in_faultstring = false;
        let mut faultcode = String::new();
        let mut faultstring = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    match local {
                        "Fault" => in_fault = true,
                        "faultcode" if in_fault => in_faultcode = true,
                        "faultstring" if in_fault => in_faultstring = true,
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_faultcode {
                        faultcode = e.unescape().unwrap_or_default().trim().to_string();
                    } else if in_faultstring {
                        faultstring = e.unescape().unwrap_or_default().trim().to_string();
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    match local {
                        "faultcode" => in_faultcode = false,
                        "faultstring" => in_faultstring = false,
                        "Fault" => {
                            if !faultstring.is_empty() {
                                let msg_lower = faultstring.to_lowercase();
                                if msg_lower.contains("invalid")
                                    && (msg_lower.contains("key")
                                        || msg_lower.contains("session")
                                        || msg_lower.contains("auth"))
                                {
                                    return Some(YukiError::AuthFailed(faultstring));
                                }
                                return Some(YukiError::SoapFault {
                                    code: faultcode,
                                    message: faultstring,
                                });
                            }
                            return None;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        None
    }
}

/// Strip any XML namespace prefix, returning only the local name.
fn local_name(name: &[u8]) -> &str {
    let s = std::str::from_utf8(name).unwrap_or("");
    s.rfind(':').map(|i| &s[i + 1..]).unwrap_or(s)
}
