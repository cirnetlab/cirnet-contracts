# Verification fixtures

## Contracts 0.1.2 parity

`contracts-0.1.2-parity.json` records validated values, the first diagnostic
and its tested fields, NCAP decoded values and errors, and sampled frame times.
The inputs for the shared cases remain in `contracts/fixtures/`. The Hardware
RPC fixture `contracts-0.1.2-rpc-v2.textproto` records the canonical
`FileDescriptorProto` with the implicit JSON field names made explicit.

The fixtures were extracted from the installed source-reference wheel built
from CIRNETLAB's Contracts 0.1.2 release revision
`524cc7a5e2e65f39077feefe70c7fb2ae3a773d6` and Git tree
`bb21f5f02c100d475ae90dfbbc96e95228066120`. The source archive SHA-256 was
`b27e02411f9c5dbd6c929907466bb190dde8fc64d36ad047bd50939e73c17dca`. The
installed source-reference wheel SHA-256 was
`f3a5c63d69fd7029ff80e612db1899b221b19d7e590ccd8595061bcecf7926cc`.
These are provenance records. Parity runs read the fixtures in this repository.
The reference fixtures retain the source project's [GPL-3.0 license](../../LICENSE).

| Fixture | SHA-256 of the LF source |
| --- | --- |
| `contracts-0.1.2-parity.json` | `868b629bb7d2f7cd1e053e162a903adfa537975efb72afd51b54b5d2179e1c45` |
| `contracts-0.1.2-rpc-v2.textproto` | `61b30b201f33488552a541eb2b4b473a779abddcbcc8d3cfc3a97a9103821d7f` |

## Historical browser projection evidence

`browser-projection-baseline.json` retains the original Remote source revision and
SHA256 values. `browser-projection-legacy.zip` stores those exact normalized-LF
projection bytes for offline verification after the CIRNET name migration. Its
OpenAPI product-info version is restored to the original 0.2.0 exactly as the
prior provenance check required. The archive is historical evidence, never input
to current generation and never a replacement baseline for the current SDK.
Current outputs are checked by deterministic generation, frozen wire constraints,
legacy/canonical reader tests and the extracted public SDK runtime probe.
