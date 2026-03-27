use crate::error::YukiError;

use super::soap_client::{SoapClient, SoapEnvelope};

const BASE_URL: &str = "https://api.yukiworks.nl/ws/AccountingInfo.asmx";

/// Client for the Yuki AccountingInfo SOAP service.
pub struct AccountingInfoClient {
    soap: SoapClient,
}

impl AccountingInfoClient {
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

    /// Retrieve full details for a single transaction by ID.
    pub async fn get_transaction_details(&self, transaction_id: &str) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("GetTransactionDetails")
            .session(session)
            .param("transactionId", transaction_id)
            .build();
        self.soap.call("GetTransactionDetails", envelope).await
    }

    /// Retrieve transactions for a GL account code over a date range.
    pub async fn get_transactions(
        &self,
        gl_account_code: &str,
        start: &str,
        end: &str,
    ) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("GetTransactions")
            .session(session)
            .param("glAccountCode", gl_account_code)
            .param("startDate", start)
            .param("endDate", end)
            .build();
        self.soap.call("GetTransactions", envelope).await
    }

    /// Retrieve the full GL account scheme.
    pub async fn get_gl_account_scheme(&self) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("GetGLAccountScheme")
            .session(session)
            .build();
        self.soap.call("GetGLAccountScheme", envelope).await
    }

    /// Retrieve the period date table for a given fiscal year.
    pub async fn get_period_date_table(&self, year: &str) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("GetPeriodDateTable")
            .session(session)
            .param("year", year)
            .build();
        self.soap.call("GetPeriodDateTable", envelope).await
    }
}

impl Default for AccountingInfoClient {
    fn default() -> Self {
        Self::new()
    }
}
