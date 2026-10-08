These certificates and the private key are public, test-only fixtures for a
loopback SOAP server. They must never be used outside tests. The leaf certificate
covers the Dutch and Belgian API hostnames; requests resolve those names to the
local listener. The test HTTP client trusts only the additional fixture CA and
keeps certificate and hostname verification enabled. Production trust is unchanged.
