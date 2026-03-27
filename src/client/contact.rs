use crate::error::YukiError;

use super::soap_client::{SoapClient, SoapEnvelope};

const BASE_URL: &str = "https://api.yukiworks.nl/ws/Contact.asmx";

/// Client for the Yuki Contact SOAP service.
pub struct ContactClient {
    soap: SoapClient,
}

impl ContactClient {
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

    /// Search for contacts matching a query string.
    pub async fn search_contacts(&self, query: &str) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("SearchContacts")
            .session(session)
            .param("searchQuery", query)
            .build();
        self.soap.call("SearchContacts", envelope).await
    }

    /// Retrieve suppliers and customers filtered by contact type.
    pub async fn get_suppliers_and_customers(
        &self,
        contact_type: &str,
    ) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("GetSuppliersAndCustomers")
            .session(session)
            .param("contactType", contact_type)
            .build();
        self.soap.call("GetSuppliersAndCustomers", envelope).await
    }
}

impl Default for ContactClient {
    fn default() -> Self {
        Self::new()
    }
}
