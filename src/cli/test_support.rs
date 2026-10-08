//! Local HTTPS SOAP fixture. No real keys, external requests, or disabled TLS checks.
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

pub(super) struct Reply {
    host: &'static str,
    service: &'static str,
    operation: &'static str,
    required: Vec<String>,
    status: u16,
    result: String,
}

impl Reply {
    pub(super) fn new(
        host: &'static str,
        service: &'static str,
        operation: &'static str,
        result: &str,
    ) -> Self {
        Self {
            host,
            service,
            operation,
            required: vec![],
            status: 200,
            result: result.into(),
        }
    }

    pub(super) fn containing(mut self, value: &str) -> Self {
        self.required.push(value.into());
        self
    }

    pub(super) fn auth(host: &'static str, service: &'static str, key: &str) -> Self {
        Self::new(host, service, "Authenticate", "session-for-test")
            .containing(&format!("<yuki:accessKey>{key}</yuki:accessKey>"))
    }

    pub(super) fn admins(host: &'static str, company: &str, domain: &str, admin: &str) -> Self {
        Self::new(host, "Accounting", "Administrations", &format!("<Administration ID=\"{admin}\"><Name>{company}</Name><DomainID>{domain}</DomainID></Administration>"))
            .containing("<yuki:sessionID>session-for-test</yuki:sessionID>")
    }

    pub(super) fn rejected(host: &'static str, key: &str) -> Self {
        let mut reply = Self::auth(host, "Accounting", key);
        reply.status = 500;
        reply.result = "<soap:Fault><faultcode>soap:Client</faultcode><faultstring>Invalid access key</faultstring></soap:Fault>".into();
        reply
    }
}

pub(super) struct SoapServer {
    pub(super) http: reqwest::Client,
    worker: JoinHandle<()>,
}

impl SoapServer {
    pub(super) fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let ca = reqwest::Certificate::from_der(include_bytes!("../../tests/fixtures/tls/ca.der"))
            .unwrap();
        let http = reqwest::Client::builder()
            .no_proxy()
            .add_root_certificate(ca)
            .resolve("api.yukiworks.be", address)
            .resolve("api.yukiworks.nl", address)
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let cert =
            CertificateDer::from(include_bytes!("../../tests/fixtures/tls/server.der").to_vec());
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            include_bytes!("../../tests/fixtures/tls/server-key.der").to_vec(),
        ));
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let tls = Arc::new(
            ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_single_cert(vec![cert], key)
                .unwrap(),
        );
        let worker = thread::spawn(move || {
            for reply in replies {
                let deadline = Instant::now() + Duration::from_secs(5);
                let socket = loop {
                    match listener.accept() {
                        Ok((socket, _)) => break socket,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(
                                Instant::now() < deadline,
                                "missing {} request to {}",
                                reply.operation,
                                reply.host
                            );
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(e) => panic!("accept: {e}"),
                    }
                };
                socket.set_nonblocking(false).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                socket
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut stream =
                    StreamOwned::new(ServerConnection::new(tls.clone()).unwrap(), socket);
                let mut header = Vec::new();
                let mut byte = [0];
                while !header.ends_with(b"\r\n\r\n") {
                    stream.read_exact(&mut byte).unwrap();
                    header.push(byte[0]);
                    assert!(header.len() < 8192);
                }
                assert_eq!(
                    stream.conn.server_name(),
                    Some(reply.host),
                    "wrong TLS server name"
                );
                let header = String::from_utf8(header).unwrap();
                assert!(
                    header.starts_with(&format!("POST /ws/{}.asmx HTTP/1.1", reply.service)),
                    "{header}"
                );
                let lower = header.to_lowercase();
                assert!(
                    lower.contains(&format!("host: {}\r\n", reply.host)),
                    "wrong Host header"
                );
                assert!(
                    header.contains(&format!(
                        "\"http://www.theyukicompany.com/{}\"",
                        reply.operation
                    )),
                    "wrong SOAPAction"
                );
                let length: usize = lower
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!(length < 65536);
                let mut body = vec![0; length];
                stream.read_exact(&mut body).unwrap();
                let body = String::from_utf8(body).unwrap();
                assert!(
                    body.contains(&format!("<yuki:{}>", reply.operation)),
                    "wrong operation"
                );
                for expected in reply.required {
                    assert!(
                        body.contains(&expected),
                        "{} missing expected test parameter {expected}",
                        reply.operation
                    );
                }
                let result = if reply.status == 200 {
                    format!(
                        "<{}Response xmlns=\"http://www.theyukicompany.com/\"><{}Result>{}</{}Result></{}Response>",
                        reply.operation,
                        reply.operation,
                        reply.result,
                        reply.operation,
                        reply.operation
                    )
                } else {
                    reply.result
                };
                let xml = format!(
                    "<soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\"><soap:Body>{result}</soap:Body></soap:Envelope>"
                );
                write!(stream, "HTTP/1.1 {} Fixture\r\nContent-Type: text/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", reply.status, xml.len(), xml).unwrap();
                stream.flush().unwrap();
            }
        });
        Self { http, worker }
    }

    pub(super) fn finish(self) {
        self.worker.join().unwrap();
    }
}
