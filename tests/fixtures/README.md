# Historical browser projection evidence

`browser-projection-baseline.json` retains the original Remote source revision and
SHA256 values. `browser-projection-legacy.zip` stores those exact normalized-LF
projection bytes for offline verification after the CIRNET name migration. Its
OpenAPI product-info version is restored to the original 0.2.0 exactly as the
prior provenance check required. The archive is historical evidence, never input
to current generation and never a replacement baseline for the current SDK.
Current outputs are checked by deterministic generation, frozen wire constraints,
legacy/canonical reader tests and the extracted public SDK runtime probe.
