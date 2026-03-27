use crate::config::Config;
use crate::error::YukiError;

pub async fn list(
    _config: &Config,
    _admin: Option<&str>,
    _folder: Option<&str>,
    _doc_type: Option<&str>,
    _format: Option<&str>,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}

pub async fn search(
    _config: &Config,
    _admin: Option<&str>,
    _query: &str,
    _format: Option<&str>,
) -> Result<(), YukiError> {
    eprintln!("not yet implemented");
    Ok(())
}
