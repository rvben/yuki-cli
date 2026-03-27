use crate::config::Config;
use crate::error::YukiError;

pub async fn list(
    _config: &Config,
    _admin: Option<&str>,
    _period: Option<&str>,
    _invoice_type: Option<&str>,
    _format: Option<&str>,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}

pub async fn show(
    _config: &Config,
    _admin: Option<&str>,
    _id: &str,
    _format: Option<&str>,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}
