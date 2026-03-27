use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::YukiError;

use super::local_name;
use super::soap_client::{SoapClient, SoapEnvelope};

const BASE_URL: &str = "https://api.yukiworks.nl/ws/Contact.asmx";

/// A Yuki contact (customer or supplier).
#[derive(Debug, Clone)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub contact_type: String,
    pub country: String,
    pub is_supplier: bool,
    pub is_customer: bool,
}

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
    pub async fn search_contacts(&self, query: &str) -> Result<Vec<Contact>, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("SearchContacts")
            .session(session)
            .param("searchQuery", query)
            .build();
        let body = self.soap.call("SearchContacts", envelope).await?;
        parse_contacts(&body)
    }

    /// Retrieve suppliers and customers filtered by contact type.
    pub async fn get_suppliers_and_customers(
        &self,
        contact_type: &str,
    ) -> Result<Vec<Contact>, YukiError> {
        let session = self.require_session()?;
        let envelope = SoapEnvelope::new("GetSuppliersAndCustomers")
            .session(session)
            .param("contactType", contact_type)
            .build();
        let body = self.soap.call("GetSuppliersAndCustomers", envelope).await?;
        parse_contacts(&body)
    }
}

/// Parse a SearchContacts or GetSuppliersAndCustomers SOAP response into a list of contacts.
///
/// Each `<Contact ID="uuid">` element carries child elements for each field.
/// The contact ID is an XML attribute; all other fields are child text nodes.
pub fn parse_contacts(xml: &str) -> Result<Vec<Contact>, YukiError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut contacts = Vec::new();
    let mut in_contact = false;
    let mut current_field = String::new();
    let mut contact = Contact {
        id: String::new(),
        name: String::new(),
        contact_type: String::new(),
        country: String::new(),
        is_supplier: false,
        is_customer: false,
    };
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = local_name(e.name().as_ref()).to_string();
                match local.as_str() {
                    "Contact" => {
                        in_contact = true;
                        contact = Contact {
                            id: String::new(),
                            name: String::new(),
                            contact_type: String::new(),
                            country: String::new(),
                            is_supplier: false,
                            is_customer: false,
                        };
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"ID" {
                                contact.id = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    "Type" | "Name" | "Country" | "IsSupplier" | "IsCustomer" if in_contact => {
                        current_field = local;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) if in_contact && !current_field.is_empty() => {
                let text = e
                    .unescape()
                    .map_err(|e| YukiError::Xml(e.to_string()))?
                    .trim()
                    .to_string();
                match current_field.as_str() {
                    "Type" => contact.contact_type = text,
                    "Name" => contact.name = text,
                    "Country" => contact.country = text,
                    "IsSupplier" => contact.is_supplier = text.eq_ignore_ascii_case("true"),
                    "IsCustomer" => contact.is_customer = text.eq_ignore_ascii_case("true"),
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    "Type" | "Name" | "Country" | "IsSupplier" | "IsCustomer" => {
                        current_field.clear();
                    }
                    "Contact" => {
                        if !contact.id.is_empty() {
                            contacts.push(contact.clone());
                        }
                        in_contact = false;
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

    Ok(contacts)
}

impl Default for ContactClient {
    fn default() -> Self {
        Self::new()
    }
}
