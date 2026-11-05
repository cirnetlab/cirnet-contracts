# Catalog authoring

English · [Tiếng Việt](../vi/catalog.md)

[Documentation index](index.md)

Edit device definitions under [device-library](../../device-library/README.md). The [physical part registry](../../device-library/parts/README.md) describes part, package and footprint revisions, pin identities and provenance. The editable JSON source separates current `parts` from immutable `historicalParts`. Increment the relevant revision when a pinout or mechanical or electrical meaning changes.

Regenerate from the Contracts repository root after a reviewed source edit.

~~~sh
python -B -m contracts.codegen.sync_editor_catalog
python -B -m contracts.codegen.sync_editor_catalog --check
python -B -m contracts.codegen.sync_breadboard_topologies --check
~~~

The generator copies JSON to the packaged `contracts/catalog-source` mirror and writes editor catalog, lean authoring descriptors and full projections under `contracts/browser/data`. Edit the source rather than the generated mirror. Installed wheels read the packaged mirror. The generator’s reported totals describe the current catalog.

Keep physical pin identity distinct from logical port names and 3D meshes. A part can support authoring and mounting before an execution profile exists. Breadboard contacts and rails derive from the declared topology. A rail has no inherent voltage. Keep saved part and package revision bindings and use explicit rebuild semantics for a newer revision. See [flexible placement](../../contracts/design-models/flexible-placement.md).
