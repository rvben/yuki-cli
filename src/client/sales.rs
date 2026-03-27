use crate::error::YukiError;

use super::soap_client::{SoapClient, SoapEnvelope};

const BASE_URL: &str = "https://api.yukiworks.nl/ws/Sales.asmx";

/// Client for the Yuki Sales SOAP service.
pub struct SalesClient {
    soap: SoapClient,
}

impl SalesClient {
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

    /// Retrieve all sales items as a raw XML body.
    pub async fn get_sales_items(&self) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("GetSalesItems").session(session).build();
        self.soap.call("GetSalesItems", envelope).await
    }
}

impl Default for SalesClient {
    fn default() -> Self {
        Self::new()
    }
}
