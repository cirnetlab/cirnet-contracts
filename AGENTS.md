# Contracts repository work

- Write each AI task's plan only in `.superpowers/plans/` and its matching spec, acceptance criteria and checklist only in `.superpowers/specs/`.
- Use `- [ ]` for open items and `- [x]` after verification. Update the affected [English](docs/en/index.md) and [Vietnamese](docs/vi/index.md) docs and the applicable [changelog](docs/changelogs/README.md).
- Verify claims against [canonical Contracts](docs/en/contract-reference.md) and distinguish tagged packages from native candidates. Preserve approved schemas, wire formats, catalog inputs, generated artifacts, fingerprints, fixtures and historical evidence.
- Contracts owns schemas, catalog sources and bindings. Remote owns FastAPI routes and system release selection. Verify the frozen OpenAPI handoff against the exact wheel.
- Follow the [verification guide](docs/en/verification.md). Allocate raw output outside every checkout, worktree and Git directory from the first write. Inspect it and clean only the owned session.
- Review Git diff, status and ignored outputs. Stage, commit or push only on request.
