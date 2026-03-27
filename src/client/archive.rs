use crate::error::YukiError;

use super::soap_client::{SoapClient, SoapEnvelope};

const BASE_URL: &str = "https://api.yukiworks.nl/ws/Archive.asmx";

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
}

impl Default for ArchiveClient {
    fn default() -> Self {
        Self::new()
    }
}
