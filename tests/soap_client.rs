use yuki_cli::client::{SoapClient, SoapEnvelope};

#[test]
fn builds_soap_envelope_with_session() {
    let envelope = SoapEnvelope::new("GLAccountBalance")
        .session("session-123")
        .param("glAccountCode", "1000")
        .param("startDate", "2025-01-01")
        .build();

    assert!(envelope.contains("GLAccountBalance"));
    assert!(envelope.contains("<yuki:sessionID>session-123</yuki:sessionID>"));
    assert!(envelope.contains("<yuki:glAccountCode>1000</yuki:glAccountCode>"));
    assert!(envelope.contains("soap:Envelope"));
    assert!(envelope.contains("http://www.theyukicompany.com/"));
}

#[test]
fn builds_soap_envelope_without_session() {
    let envelope = SoapEnvelope::new("Authenticate")
        .param("accessKey", "my-api-key")
        .build();

    assert!(envelope.contains("Authenticate"));
    assert!(envelope.contains("<yuki:accessKey>my-api-key</yuki:accessKey>"));
    assert!(!envelope.contains("sessionID"));
}

#[test]
fn parses_authenticate_response() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
               xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <soap:Body>
    <AuthenticateResponse xmlns="http://www.theyukicompany.com/">
      <AuthenticateResult>session-id-abc-123</AuthenticateResult>
    </AuthenticateResponse>
  </soap:Body>
</soap:Envelope>"#;

    let result = SoapClient::parse_single_result(xml, "AuthenticateResult").unwrap();
    assert_eq!(result, "session-id-abc-123");
}

#[test]
fn parses_soap_fault() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
  <soap:Body>
    <soap:Fault>
      <faultcode>soap:Client</faultcode>
      <faultstring>Invalid API key</faultstring>
    </soap:Fault>
  </soap:Body>
</soap:Envelope>"#;

    let err = SoapClient::parse_single_result(xml, "AuthenticateResult");
    assert!(err.is_err());
    let err_msg = format!("{}", err.unwrap_err());
    assert!(err_msg.contains("Invalid API key"));
}

#[test]
fn soap_action_header_format() {
    let action = SoapClient::soap_action("Accounting", "GLAccountBalance");
    assert_eq!(action, "http://www.theyukicompany.com/GLAccountBalance");
}
