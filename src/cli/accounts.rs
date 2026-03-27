use crate::config::Config;
use crate::error::YukiError;

pub async fn balance(
    _config: &Config,
    _admin: Option<&str>,
    _account: &str,
    _period: Option<&str>,
    _format: Option<&str>,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}

pub async fn transactions(
    _config: &Config,
    _admin: Option<&str>,
    _account: &str,
    _period: Option<&str>,
    _format: Option<&str>,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}
