use sha2::{Digest, Sha256};
use std::error::Error;
use std::path::{Path, PathBuf};

fn protoc() -> Result<PathBuf, Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=CIRNET_PROTOC_SHA256");
    match (
        std::env::var_os("PROTOC"),
        std::env::var_os("CIRNET_PROTOC_SHA256"),
    ) {
        (None, None) => Ok(protoc_bin_vendored::protoc_bin_path()?),
        (Some(path), Some(expected)) => {
            let path = PathBuf::from(path);
            if !path.is_absolute() || !path.is_file() {
                return Err("PROTOC must name an absolute compiler file".into());
            }
            println!("cargo:rerun-if-changed={}", path.display());
            let actual = format!("{:x}", Sha256::digest(std::fs::read(&path)?));
            if expected != actual.as_str() {
                return Err("PROTOC SHA-256 differs from CIRNET_PROTOC_SHA256".into());
            }
            Ok(path)
        }
        _ => Err("Set both PROTOC and CIRNET_PROTOC_SHA256 to select a compiler".into()),
    }
}

fn descriptor(
    directory: &Path,
    output: &Path,
    protoc: &Path,
) -> Result<prost_types::FileDescriptorSet, Box<dyn Error>> {
    let schema = directory.join("rpc_v2.proto");
    println!("cargo:rerun-if-changed={}", schema.display());
    Ok(prost_build::Config::new()
        .protoc_executable(protoc)
        .file_descriptor_set_path(output)
        .skip_source_info()
        .load_fds(&[schema], &[directory])?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let output = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("Cargo did not set OUT_DIR")?);
    let protoc = protoc()?;
    let canonical = descriptor(
        Path::new("contracts/hardware/v2"),
        &output.join("hardware-v2.pb"),
        &protoc,
    )?;
    descriptor(
        Path::new("contracts/compatibility/2026-10-16/hardware/v2"),
        &output.join("hardware-v2-legacy.pb"),
        &protoc,
    )?;
    prost_build::Config::new()
        .btree_map(["."])
        .compile_fds(canonical)?;
    Ok(())
}
