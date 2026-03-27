use crate::config::Config;
use crate::error::YukiError;

pub async fn btw(
    _config: &Config,
    _admin: Option<&str>,
    _period: Option<&str>,
    _format: Option<&str>,
    _quiet: bool,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}

pub async fn jaarwerk(
    _config: &Config,
    _admin: Option<&str>,
    _year: Option<&str>,
    _format: Option<&str>,
    _quiet: bool,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}
