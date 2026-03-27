use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::YukiError;

use super::local_name;
use super::soap_client::{SoapClient, SoapEnvelope};

const BASE_URL: &str = "https://api.yukiworks.nl/ws/Archive.asmx";

/// A Yuki cost category.
#[derive(Debug, Clone)]
pub struct CostCategory {
    pub id: String,
    pub description: String,
}

/// A Yuki payment method.
#[derive(Debug, Clone)]
pub struct PaymentMethod {
    pub id: String,
    pub description: String,
}

/// Client for the Yuki Archive SOAP service.
pub struct ArchiveClient {
    soap: SoapClient,
}

impl ArchiveClient {
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

    /// List all documents in an archive folder by folder ID.
    pub async fn documents_in_folder(&self, folder_id: i32) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("DocumentsInFolder")
            .session(session)
            .param("folderID", &folder_id.to_string())
            .build();
        self.soap.call("DocumentsInFolder", envelope).await
    }

    /// List all documents of a given document type.
    pub async fn documents_by_type(&self, doc_type: i32) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("DocumentsByType")
            .session(session)
            .param("documentType", &doc_type.to_string())
            .build();
        self.soap.call("DocumentsByType", envelope).await
    }

    /// Search documents using a free-text query.
    pub async fn search_documents(&self, query: &str) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("SearchDocuments")
            .session(session)
            .param("searchText", query)
            .build();
        self.soap.call("SearchDocuments", envelope).await
    }

    /// List documents of a given type that were modified since the specified date.
    pub async fn modified_documents_by_type(
        &self,
        doc_type: i32,
        modified_since: &str,
    ) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("ModifiedDocumentsByType")
            .session(session)
            .param("documentType", &doc_type.to_string())
            .param("modifiedSince", modified_since)
            .build();
        self.soap.call("ModifiedDocumentsByType", envelope).await
    }

    /// Upload a document to the archive without additional metadata.
    ///
    /// Returns the document ID assigned by Yuki.
    pub async fn upload_document(
        &self,
        admin_id: &str,
        filename: &str,
        data_base64: &str,
        folder_id: i32,
    ) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("UploadDocument")
            .session(session)
            .param("fileName", filename)
            .param("data", data_base64)
            .param("folder", &folder_id.to_string())
            .param("administrationID", admin_id)
            .build();
        let body = self.soap.call("UploadDocument", envelope).await?;
        SoapClient::parse_single_result(&body, "UploadDocumentResult")
    }

    /// Upload a document to the archive with invoice metadata.
    ///
    /// Returns the document ID assigned by Yuki.
    #[allow(clippy::too_many_arguments)]
    pub async fn upload_document_with_data(
        &self,
        admin_id: &str,
        filename: &str,
        data_base64: &str,
        folder_id: i32,
        currency: &str,
        amount: f64,
        cost_category: Option<&str>,
        payment_method: Option<&str>,
        project: Option<&str>,
        remarks: Option<&str>,
    ) -> Result<String, YukiError> {
        let session = self.require_session()?;
        let amount_str = format!("{amount:.2}");
        let envelope = SoapEnvelope::new("UploadDocumentWithData")
            .session(session)
            .param("fileName", filename)
            .param("data", data_base64)
            .param("folder", &folder_id.to_string())
            .param("administrationID", admin_id)
            .param("currency", currency)
            .param("amount", &amount_str)
            .param("costCategory", cost_category.unwrap_or(""))
            .param("paymentMethod", payment_method.unwrap_or(""))
            .param("project", project.unwrap_or(""))
            .param("remarks", remarks.unwrap_or(""))
            .build();
        let body = self.soap.call("UploadDocumentWithData", envelope).await?;
        SoapClient::parse_single_result(&body, "UploadDocumentWithDataResult")
    }

    /// Retrieve all available cost categories.
    pub async fn cost_categories(&self) -> Result<Vec<CostCategory>, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("CostCategories").session(session).build();
        let body = self.soap.call("CostCategories", envelope).await?;
        Self::parse_cost_categories(&body)
    }

    /// Retrieve all available payment methods.
    pub async fn payment_methods(&self) -> Result<Vec<PaymentMethod>, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("PaymentMethods").session(session).build();
        let body = self.soap.call("PaymentMethods", envelope).await?;
        Self::parse_payment_methods(&body)
    }

    /// Parse a CostCategories SOAP response.
    ///
    /// Each `CostCategory` element carries an `ID` attribute and a `Description` child element:
    /// `<CostCategory ID="45100"><Description>...</Description></CostCategory>`
    fn parse_cost_categories(xml: &str) -> Result<Vec<CostCategory>, YukiError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut categories = Vec::new();
        let mut current_id = String::new();
        let mut current_desc = String::new();
        let mut in_category = false;
        let mut in_description = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = local_name(e.name().as_ref()).to_string();
                    match local.as_str() {
                        "CostCategory" => {
                            in_category = true;
                            current_id.clear();
                            current_desc.clear();
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"ID" {
                                    current_id = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                        }
                        "Description" if in_category => {
                            in_description = true;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) if in_description => {
                    current_desc = e
                        .unescape()
                        .map_err(|e| YukiError::Xml(e.to_string()))?
                        .trim()
                        .to_string();
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    match local {
                        "Description" => in_description = false,
                        "CostCategory" => {
                            if !current_id.is_empty() {
                                categories.push(CostCategory {
                                    id: current_id.clone(),
                                    description: current_desc.clone(),
                                });
                            }
                            in_category = false;
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

        Ok(categories)
    }

    /// Parse a PaymentMethods SOAP response.
    ///
    /// Each `PaymentMethod` element carries an `ID` attribute and a `Description` child element:
    /// `<PaymentMethod ID="4"><Description>...</Description></PaymentMethod>`
    fn parse_payment_methods(xml: &str) -> Result<Vec<PaymentMethod>, YukiError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut methods = Vec::new();
        let mut current_id = String::new();
        let mut current_desc = String::new();
        let mut in_method = false;
        let mut in_description = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = local_name(e.name().as_ref()).to_string();
                    match local.as_str() {
                        "PaymentMethod" => {
                            in_method = true;
                            current_id.clear();
                            current_desc.clear();
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"ID" {
                                    current_id = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                        }
                        "Description" if in_method => {
                            in_description = true;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) if in_description => {
                    current_desc = e
                        .unescape()
                        .map_err(|e| YukiError::Xml(e.to_string()))?
                        .trim()
                        .to_string();
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    match local {
                        "Description" => in_description = false,
                        "PaymentMethod" => {
                            if !current_id.is_empty() {
                                methods.push(PaymentMethod {
                                    id: current_id.clone(),
                                    description: current_desc.clone(),
                                });
                            }
                            in_method = false;
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

        Ok(methods)
    }
}

impl Default for ArchiveClient {
    fn default() -> Self {
        Self::new()
    }
}
