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
classifier names and exact I/O prefix. Explicit trust settings are unchanged.
The dependency floor is rustls 0.23.46. No audit exception is added.

Run tests through the Toadsworth client repository and its pinned git
revision. Do not invoke cb from this fork: cb supports only the client and
control-plane repositories. Live SQL Server tests and their isolated
container runner are in the client repository under TOAD-90.
