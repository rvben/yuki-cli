//! Check actual CLI routing without sending credentials to any external service.
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

fn requested_host(args: &[&str], config: Option<&str>) -> String {
    let home = tempfile::TempDir::new().unwrap();
    if let Some(config) = config {
        let path = home.path().join(".config/yuki/config.toml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, config).unwrap();
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let proxy = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "CLI never contacted the proxy");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("accept failed: {e}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        let mut byte = [0];
        while !request.ends_with(b"\r\n\r\n") {
            assert_eq!(stream.read(&mut byte).unwrap(), 1);
            request.push(byte[0]);
            assert!(request.len() < 8192);
        }
        // Refuse the tunnel: the test captures only its destination, before TLS or credentials.
        stream
            .write_all(
                b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        String::from_utf8(request)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .to_owned()
    });
    let output = Command::new(env!("CARGO_BIN_EXE_yuki"))
        .args(args)
        .env("HOME", home.path())
        .env("HTTPS_PROXY", &proxy)
        .env("https_proxy", &proxy)
        .env("ALL_PROXY", &proxy)
        .env("all_proxy", &proxy)
        .env("NO_PROXY", "")
        .env("no_proxy", "")
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "the test proxy rejects all requests"
    );
    let path = home.path().join(".config/yuki/config.toml");
    match config {
        Some(original) => assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            original,
            "failed authentication must preserve configuration"
        ),
        None => assert!(
            !path.exists(),
            "failed authentication must not create configuration"
        ),
    }
    server.join().unwrap()
}

const MIXED_CONFIG: &str = r#"api_key = "fake-nl-key"
default_admin = "nl"
[administrations.nl]
domain_id = "domain-nl"
admin_id = "admin-nl"
[administrations.be]
domain_id = "domain-be"
admin_id = "admin-be"
api_key = "fake-be-key"
region = "be"
"#;

#[test]
fn setup_uses_explicit_region_and_reuses_saved_region() {
    for prefix in [vec!["init"], vec!["auth", "login"]] {
        let mut args = prefix.clone();
        args.extend(["--region", "be", "--api-key", "fake-key"]);
        assert_eq!(
            requested_host(&args, None),
            "CONNECT api.yukiworks.be:443 HTTP/1.1"
        );
        let saved = format!("region = 'be'\n{MIXED_CONFIG}");
        let mut args = prefix;
        args.extend(["--api-key", "fake-key"]);
        assert_eq!(
            requested_host(&args, Some(&saved)),
            "CONNECT api.yukiworks.be:443 HTTP/1.1"
        );
    }
    assert_eq!(
        requested_host(
            &["init", "--add", "--region", "be", "--api-key", "fake-key"],
            Some(MIXED_CONFIG)
        ),
        "CONNECT api.yukiworks.be:443 HTTP/1.1"
    );
}

#[test]
fn commands_route_to_selected_profile_region_across_all_services() {
    for command in [
        vec!["doctor"],
        vec!["auth", "status"],
        vec!["accounts", "scheme"],
        vec!["vat", "codes"],
        vec!["contacts", "list"],
        vec!["documents", "search", "invoice"],
        vec!["upload", "categories"],
        vec!["invoices", "list", "--invoice-type", "sales"],
        vec!["projects", "list"],
        vec!["check", "unmatched", "--period", "2026-Q1"],
    ] {
        let mut be = command.clone();
        be.extend(["--profile", "be"]);
        assert_eq!(
            requested_host(&be, Some(MIXED_CONFIG)),
            "CONNECT api.yukiworks.be:443 HTTP/1.1",
            "{be:?}"
        );
    }
    assert_eq!(
        requested_host(&["doctor"], Some(MIXED_CONFIG)),
        "CONNECT api.yukiworks.nl:443 HTTP/1.1"
    );
    assert_eq!(
        requested_host(
            &["admin", "list"],
            Some(&format!("region = 'be'\n{MIXED_CONFIG}"))
        ),
        "CONNECT api.yukiworks.be:443 HTTP/1.1"
    );
}
