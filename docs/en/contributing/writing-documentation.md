# Maintain Contracts documentation

[Tiếng Việt](../../vi/contributing/writing-documentation.md) · [Documentation](../index.md) · [Repository instructions](../../../AGENTS.md)

Keep the root README in English and link to both documentation indexes. Keep current guides at matching relative paths under `docs/en/` and `docs/vi/`. Update the pair together with the same identifiers, examples, limits and capability claims. Contracts owns schemas, wire formats, catalogs and generated bindings. Remote owns FastAPI route registration and system release selection. [Contract reference](../contract-reference.md)

Use [changelogs](../../changelogs/README.md) for verified version changes. Keep one `CHANGELOG_V<major>.md` for each released major, with Unreleased first and published versions in descending order. Give each release a SemVer heading, a link in the contents, an evidenced date and a tag target. Distinguish current source, verified candidates and published artifacts when describing release status. [Verification evidence](../native.md#verification-evidence)

For every AI task, keep the plan in `.superpowers/plans/` and the matching spec, acceptance criteria and checklist in `.superpowers/specs/`. Both directories are already ignored. Use `- [ ]` for open work and `- [x]` after verification.

Preserve the [original verification and migration records](../history.md) at their recorded paths and in their source language. Preserve the frozen OpenAPI handoff, schema and wire baselines, fixtures and resource fingerprints. Before moving or retiring a document, trace Markdown, package, build, CI and test references and confirm the change is authorized.

Check each affected language pair and verify local paths, case, anchors and cross-repository references. Keep raw evidence in an allocator-owned session outside checkouts, worktrees and Git directories. After verification, update the relevant technical pair and changelog, then review Git diff, status and ignored output. [Verification guide](../verification.md)
