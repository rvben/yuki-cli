use crate::config::Config;
use crate::error::YukiError;

pub async fn returns(
    _config: &Config,
    _admin: Option<&str>,
    _year: Option<&str>,
    _format: Option<&str>,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}

pub async fn codes(
    _config: &Config,
    _admin: Option<&str>,
    _format: Option<&str>,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}
