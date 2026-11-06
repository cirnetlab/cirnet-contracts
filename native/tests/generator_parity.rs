use cirnet_contracts_native::generator::generate;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn copy_sources(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let path = entry.path();
        if path.is_dir() {
            if !matches!(
                name.to_str(),
                Some("browser" | "catalog-source" | "boundary" | "__pycache__")
            ) {
                copy_sources(&path, &to.join(name));
            }
        } else if !matches!(
            name.to_str(),
            Some("fingerprints.json" | "physical-parts.v1.json" | "signal-port-catalog.json")
        ) && (matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("json" | "proto" | "mjs")
        ) || name == "capture_codec.py")
        {
            fs::copy(path, to.join(name)).unwrap();
        }
    }
}

fn json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn sdk_inputs() -> (PathBuf, tempfile::TempDir) {
    let node_tools = std::env::var_os("CIRNET_SDK_NODE_TOOLS")
        .expect("CIRNET_SDK_NODE_TOOLS must point to the locked Node tools");
    let scratch = std::env::var_os("CIRNET_SDK_TEST_OUTPUT")
        .expect("CIRNET_SDK_TEST_OUTPUT must point to an owned external directory");
    let run = tempfile::Builder::new()
        .prefix("sdk-regeneration-")
        .tempdir_in(scratch)
        .unwrap();
    (node_tools.into(), run)
}

fn copy_sdk_sources(run: &Path) -> PathBuf {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = run.join("source");
    copy_sources(&repo.join("contracts"), &source.join("contracts"));
    copy_sources(&repo.join("device-library"), &source.join("device-library"));
    source
}

#[test]
#[ignore = "requires CIRNET_SDK_NODE_TOOLS and CIRNET_SDK_TEST_OUTPUT"]
fn generates_sdk_projections_from_sources_without_python_or_browser_snapshots() {
    let (node_tools, run) = sdk_inputs();
    let run = run.keep();
    let source = copy_sdk_sources(&run);
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));

    let first = run.join("first");
    generate(&source, &first, &node_tools, None).unwrap();
    let browser = first.join("contracts/browser");
    for relative in [
        "schemas/structural.json",
        "schemas/editor.json",
        "schemas/transport.json",
        "schemas/profiles.json",
        "data/breadboardTopologies.json",
        "data/editorCatalog.json",
        "data/editorParts.json",
        "data/physicalParts.json",
        "data/signalPortCatalog.json",
        "data/labContracts.json",
        "data/flexibleMechanics.json",
        "data/breadboardMechanics.json",
        "fixtures/conformance.json",
        "fixtures/capture-conformance.json",
    ] {
        assert_eq!(
            json(&browser.join(relative)),
            json(&repo.join("contracts/browser").join(relative)),
            "{relative}"
        );
    }
    for relative in [
        "operations.ts",
        "schemaAliases.ts",
        "dto.ts",
        "captureLayout.ts",
        "validators.js",
    ] {
        assert_eq!(
            fs::read(browser.join("generated").join(relative)).unwrap(),
            fs::read(repo.join("contracts/browser/generated").join(relative)).unwrap(),
            "{relative}"
        );
    }

    let canonical_layout = source.join("contracts/capture/layout.json");
    let layout = fs::read_to_string(&canonical_layout).unwrap();
    assert!(layout.contains("65536"));
    fs::write(&canonical_layout, layout.replace("65536", "65535")).unwrap();
    let second = run.join("second");
    generate(&source, &second, &node_tools, None).unwrap();
    let projected =
        fs::read_to_string(second.join("contracts/browser/generated/captureLayout.ts")).unwrap();
    assert!(projected.contains(r#""max_header_bytes": 65535"#));
    let original = json(&first.join("contracts/fingerprints.json"));
    assert_eq!(original, json(&repo.join("contracts/fingerprints.json")));
    let modified = json(&second.join("contracts/fingerprints.json"));
    assert_ne!(
        original["canonical_source_sha256"],
        modified["canonical_source_sha256"]
    );
    assert_eq!(
        original["public_schema_sha256"],
        modified["public_schema_sha256"]
    );
    assert_eq!(original["openapi_sha256"], modified["openapi_sha256"]);
}

fn assert_generation_rejected(source: &Path, run: &Path, node_tools: &Path, expected: &str) {
    let output = run.join("candidate");
    let error = generate(source, &output, node_tools, None)
        .expect_err("malformed source was published")
        .to_string();
    assert!(error.contains(expected), "{error}");
    assert!(!output.exists());
}

#[test]
#[ignore = "requires CIRNET_SDK_NODE_TOOLS and CIRNET_SDK_TEST_OUTPUT"]
fn rejects_malformed_canonical_catalog_before_publication() {
    let (node_tools, run) = sdk_inputs();
    let source = copy_sdk_sources(run.path());
    let file = source.join("device-library/parts/physical-parts.json");
    let original = json(&file);
    for (pointer, value, expected) in [
        ("/parts/0/package/bodyMm/x", json!(0), "body dimensions"),
        ("/parts/0/revision", json!(0), "positive revisions"),
        ("/parts/0/package/pins/0/id", json!(""), "pin identity"),
        (
            "/parts/0/package/shape/family",
            json!("led"),
            "package shape",
        ),
    ] {
        let mut modified = original.clone();
        *modified.pointer_mut(pointer).unwrap() = value;
        fs::write(&file, serde_json::to_vec(&modified).unwrap()).unwrap();
        assert_generation_rejected(&source, run.path(), &node_tools, expected);
    }
    let mut duplicate = original.clone();
    duplicate["historicalParts"]
        .as_array_mut()
        .unwrap()
        .push(original["parts"][0].clone());
    fs::write(&file, serde_json::to_vec(&duplicate).unwrap()).unwrap();
    assert_generation_rejected(
        &source,
        run.path(),
        &node_tools,
        "Duplicate physical part identity",
    );
    fs::write(&file, serde_json::to_vec(&original).unwrap()).unwrap();

    let file = source.join("contracts/design-models/flexible-mechanics.json");
    let mut mechanics = json(&file);
    let profiles = mechanics["profiles"].as_array_mut().unwrap();
    profiles.push(profiles[0].clone());
    fs::write(&file, serde_json::to_vec(&mechanics).unwrap()).unwrap();
    assert_generation_rejected(
        &source,
        run.path(),
        &node_tools,
        "Ambiguous mechanical package profile",
    );
}

#[test]
#[ignore = "requires CIRNET_SDK_NODE_TOOLS and CIRNET_SDK_TEST_OUTPUT"]
fn rejects_invalid_manifest_and_unmanifested_schema() {
    let (node_tools, run) = sdk_inputs();
    let source = copy_sdk_sources(run.path());
    let file = source.join("contracts/manifest.json");
    let original = json(&file);
    for (key, value, expected) in [
        ("kind", json!("unrecognized"), "Unknown contract kind"),
        ("visibility", json!("secret"), "Unknown contract visibility"),
        ("producer", json!(""), "Incomplete contract ownership"),
    ] {
        let mut modified = original.clone();
        let entry = modified["contracts"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["kind"] == "data")
            .unwrap();
        entry[key] = value;
        fs::write(&file, serde_json::to_vec(&modified).unwrap()).unwrap();
        assert_generation_rejected(&source, run.path(), &node_tools, expected);
    }
    fs::write(&file, serde_json::to_vec(&original).unwrap()).unwrap();
    fs::write(source.join("contracts/unmanifested.schema.json"), "{}").unwrap();
    assert_generation_rejected(&source, run.path(), &node_tools, "Unmanifested schema");
}

#[test]
#[ignore = "requires CIRNET_SDK_NODE_TOOLS and CIRNET_SDK_TEST_OUTPUT"]
fn rejects_canonical_directory_linked_outside_source() {
    let (node_tools, run) = sdk_inputs();
    let source = copy_sdk_sources(run.path());
    let directory = source.join("device-library");
    let external = run.path().join("outside-library");
    fs::rename(&directory, &external).unwrap();
    #[cfg(windows)]
    {
        let result = Command::new("cmd.exe")
            .current_dir(run.path())
            .args([
                "/c",
                "mklink",
                "/J",
                "source\\device-library",
                "outside-library",
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&external, &directory).unwrap();
    assert_generation_rejected(
        &source,
        run.path(),
        &node_tools,
        "escapes canonical source root",
    );
}

#[test]
#[ignore = "requires CIRNET_SDK_NODE_TOOLS and CIRNET_SDK_TEST_OUTPUT"]
fn sdk_pack_cannot_reuse_stale_staging() {
    let (node_tools, run) = sdk_inputs();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let stage = run.path().join("sdk-source/dist");
    fs::create_dir_all(&stage).unwrap();
    fs::write(stage.join("stale.js"), "export const stale = true").unwrap();
    let result = Command::new("node")
        .arg(repo.join("scripts/check_public_sdk.mjs"))
        .arg("--output")
        .arg(run.path())
        .arg("--node-tools")
        .arg(node_tools)
        .env(
            "CIRNET_ARTIFACT_PATHS_MODULE",
            url::Url::from_file_path(
                repo.join("tools/artifact-paths/cirnet_artifact_paths/artifact-paths.mjs"),
            )
            .unwrap()
            .as_str(),
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let archive = run.path().join(format!(
        "cirnet-contracts-{}.tgz",
        env!("CARGO_PKG_VERSION")
    ));
    let files = Command::new("tar")
        .arg("-tzf")
        .arg(archive)
        .output()
        .unwrap();
    assert!(
        files.status.success(),
        "{}",
        String::from_utf8_lossy(&files.stderr)
    );
    assert!(!String::from_utf8_lossy(&files.stdout).contains("stale.js"));
}

#[test]
fn refuses_to_generate_over_the_source_tree() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let error = generate(repo, repo, repo, None).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("fresh external output directory"),
        "{error}"
    );
}
