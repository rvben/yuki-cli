use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::YukiError;

use super::local_name;
use super::soap_client::{SoapClient, SoapEnvelope};

const BASE_URL: &str = "https://api.yukiworks.nl/ws/AccountingInfo.asmx";

/// Full details for a single transaction line.
#[derive(Debug, Clone)]
pub struct TransactionDetail {
    pub id: String,
    pub date: String,
    pub description: String,
    pub amount: String,
    pub currency: String,
    pub gl_account_code: String,
}

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
    pub async fn get_transaction_details(
        &self,
        transaction_id: &str,
    ) -> Result<Vec<TransactionDetail>, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("GetTransactionDetails")
            .session(session)
            .param("transactionId", transaction_id)
            .build();
        let body = self.soap.call("GetTransactionDetails", envelope).await?;
        Self::parse_transaction_details(&body)
    }

    /// Parse a GetTransactionDetails SOAP response into a list of `TransactionDetail` values.
    ///
    /// Each `TransactionInfo` element carries child elements `id`, `transactionDate`,
    /// `description`, `transactionAmount`, `currency`, and `glAccountCode`.
    pub fn parse_transaction_details(xml: &str) -> Result<Vec<TransactionDetail>, YukiError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut details = Vec::new();
        let mut in_info = false;
        let mut field: Option<String> = None;
        let mut current = TransactionDetail {
            id: String::new(),
            date: String::new(),
            description: String::new(),
            amount: String::new(),
            currency: String::new(),
            gl_account_code: String::new(),
        };
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = local_name(e.name().as_ref()).to_string();
                    match local.as_str() {
                        "TransactionInfo" => {
                            in_info = true;
                            current = TransactionDetail {
                                id: String::new(),
                                date: String::new(),
                                description: String::new(),
                                amount: String::new(),
                                currency: String::new(),
                                gl_account_code: String::new(),
                            };
                        }
                        "id" | "transactionDate" | "description" | "transactionAmount"
                        | "currency" | "glAccountCode"
                            if in_info =>
                        {
                            field = Some(local);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref f) = field {
                        let text = e
                            .unescape()
                            .map_err(|e| YukiError::Xml(e.to_string()))?
                            .trim()
                            .to_string();
                        match f.as_str() {
                            "id" => current.id = text,
                            "transactionDate" => current.date = text,
                            "description" => current.description = text,
                            "transactionAmount" => current.amount = text,
                            "currency" => current.currency = text,
                            "glAccountCode" => current.gl_account_code = text,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = local_name(e.name().as_ref()).to_string();
                    match local.as_str() {
                        "id" | "transactionDate" | "description" | "transactionAmount"
                        | "currency" | "glAccountCode" => {
                            field = None;
                        }
                        "TransactionInfo" if in_info => {
                            details.push(current.clone());
                            in_info = false;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(YukiError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(details)
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
