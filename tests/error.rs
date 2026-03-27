#[test]
fn auth_failed_returns_exit_code_2() {
    let err = yuki_cli::error::YukiError::AuthFailed("bad key".into());
    assert_eq!(err.exit_code(), 2);
}

#[test]
fn not_found_returns_exit_code_3() {
    let err = yuki_cli::error::YukiError::NotFound("missing".into());
    assert_eq!(err.exit_code(), 3);
}

#[test]
fn rate_limited_returns_exit_code_4() {
    let err = yuki_cli::error::YukiError::RateLimited;
    assert_eq!(err.exit_code(), 4);
}

#[test]
fn soap_fault_returns_exit_code_1() {
    let err = yuki_cli::error::YukiError::SoapFault {
        code: "Server".into(),
        message: "internal".into(),
    };
    assert_eq!(err.exit_code(), 1);
}

#[test]
fn config_error_returns_exit_code_1() {
    let err = yuki_cli::error::YukiError::Config("missing file".into());
    assert_eq!(err.exit_code(), 1);
}

#[test]
fn xml_error_returns_exit_code_1() {
    let err = yuki_cli::error::YukiError::Xml("parse failed".into());
    assert_eq!(err.exit_code(), 1);
}
