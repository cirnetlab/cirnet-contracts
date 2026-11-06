use std::error::Error;
use std::path::{Path, PathBuf};

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
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
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
