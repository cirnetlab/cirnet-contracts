# Contracts repository work

- Contracts owns canonical schemas, CircuitGraph, catalog input, NCAP, Hardware RPC types, bindings and native resources. Remote owns operation registration and release selection. Verify the OpenAPI handoff against the exact exported artifact.
- Preserve schemas, catalogs, wire formats, fixture bytes, compatibility fingerprints and authority boundaries. Keep production runtime independent of the historical Python implementation.
- Keep task plans in `.superpowers/plans/` and specifications, acceptance criteria and checklists in `.superpowers/specs/`. Mark items `- [ ]` until verified.
- Search for existing documentation first. Update the authoritative page, consolidate overlap and verify both the [English](docs/en/index.md) and [Vietnamese](docs/vi/index.md) versions. Update the applicable [changelog](docs/changelogs/README.md).
- Distinguish `TARGET`, `IMPLEMENTED`, `TESTED`, `VERIFIED`, `RELEASED`, `UNAVAILABLE` and `BLOCKED`. Use an owned external session for raw output and retain approved summaries only.
- Follow the [verification guide](docs/en/verification.md). Review Git diff, status and ignored outputs. Stage, commit or publish only on request.
