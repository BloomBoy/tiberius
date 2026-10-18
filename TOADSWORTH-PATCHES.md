# Toadsworth 0.13 port (TOAD-90)

Base: upstream v0.13.0, `7fd75bb7070fe6ea41374698abf78928bb983bda`.
Previous fork: `013644e4ee8b313f8440e3bde49443727f7829c4`.

| Previous commit | Behavior retained in this port |
| --- | --- |
| 614aeb8 | Configured LOGIN7 packet size; already present upstream. |
| 107746b | `Client::packet_size()` reports the size negotiated by the server. |
| 002f804 | Explicit bulk column order, omitted defaults/identity values, TABLOCK, and early named-column refusal. Uses 0.13's `SqlBulkCopyOptions` API; restores empty-list behavior and the existing refusal text. |
| e37cfdc | DATE TYPE_INFO has no extra length byte; already fixed and tested upstream. |
| f479b91 | TIME/DATETIMEOFFSET scales and Unicode character widths in INSERT BULK. Identifier quoting is already present upstream. |
| 013644e | Nullable SMALLDATETIME distinguished by length. MONEY/SMALLMONEY declarations are already fixed upstream; the application's existing fallback remains in place. |

The earlier nine commits between v0.12.2 and 0e2897a are upstream work:
chrono oldtime removal, encoding_rs, ReadOnlyIntent, row iteration,
tokio-rustls update, empty variable-length encoding, formatting/tests,
and column type information. Their supported behavior remains in 0.13;
the old TLS dependency is deliberately replaced.

`toadsworth-compat` preserves the shipped rustls connection contract:
`trust_cert_ca` replaces the prior file, trusts exactly its one PEM/CRT/DER
certificate without adding system roots, and preserves CA loading errors.
With no named CA, system roots still apply. Certificate validation still
runs on the same paths; certificate refusals retain the application's
classifier names and exact I/O prefix. This includes rustls 0.21's BadSignature for
unsupported or mismatched signature algorithms (e.g. a SHA-1 signed server
certificate) and Other(RequiredEkuNotFound) for a certificate without the
serverAuth EKU. Explicit trust settings are unchanged.
The dependency floor is rustls 0.23.46. No audit exception is added.

`toadsworth-compat` also keeps the shipped deadlines: 0.13 adds a 15-second
handshake timeout and a 30-second command timeout by default, which the
0.12.2 fork did not have. Under the feature both default to `None`, so a slow
first response still waits on a usable session and the application's own
30-second connect limit, with its message, remains the only login bound.
Explicit `handshake_timeout`/`command_timeout` settings still apply.

Run tests through the Toadsworth client repository and its pinned git
revision. Do not invoke cb from this fork: cb supports only the client and
control-plane repositories. Live SQL Server tests and their isolated
container runner are in the client repository under TOAD-90.

`trust_cert_ca_pem` (Toadsworth) takes a named CA as PEM text, for
connections whose CA arrives as text rather than a path (an organisation
register entry). Under `toadsworth-compat` it follows `trust_cert_ca`: it
replaces the previous CA and the system roots and must hold exactly one
certificate. Unparseable text is refused as "Could not read provided CA
certificate! The PEM text is not a valid certificate." and never echoed.
