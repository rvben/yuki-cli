# yuki-client

Typed async client for the [Nmbrs Accounting](https://www.nmbrs.com/nl/accounting/nmbrs-accounting) (formerly Yuki) SOAP API.

The crate retains its `yuki-client` name for compatibility. The existing SOAP namespace remains unchanged. Clients default to the Dutch
`api.yukiworks.nl` host; use `with_region(Region::Be)` for the Belgian
`api.yukiworks.be` host, or `with_region_and_client` to reuse a pooled HTTP client.

```rust
use yuki_client::{Region, client::accounting::AccountingClient};

let client = AccountingClient::with_region(Region::Be);
```

This crate is the transport and parsing layer extracted from
[`yuki-cli`](https://github.com/rvben/yuki-cli): SOAP envelope building,
key-based authentication, and typed wrappers over the Accounting, AccountingInfo,
Sales, VAT, Contact, and Archive services. The CLI is a thin presentation layer
on top; other consumers (such as a long-running service) can depend on this crate
directly.

## HTTP client

`SoapClient::new` builds its own `reqwest::Client`, which is convenient for
short-lived processes. Long-running consumers should pass a shared, pooled client
via `SoapClient::with_client` (and the matching `XClient::with_client`
constructors) so connections are reused across requests.

## Yuki API notes

- SOAP namespace: `http://www.theyukicompany.com/`
- Flow is always `authenticate()` -> `set_current_domain()` -> operation calls.
- SOAP faults are returned as HTTP 500; the body is parsed for the fault.
- Sessions expire quickly, so each run authenticates fresh.

## License

MIT
