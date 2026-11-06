use cirnet_contracts_native::generator;
use cirnet_contracts_native::hardware_v2::{FILE_DESCRIPTOR_SET, LEGACY_FILE_DESCRIPTOR_SET};
use cirnet_contracts_native::validation::{Registry, ValidationError};
use cirnet_contracts_native::wire_json;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str =
    "cirnet-contracts generate-sdk <source-root> <output-root> <node-tools> [openapi]
cirnet-contracts validate <source-root> <contract> <profile>
cirnet-contracts descriptor [legacy]";

fn validate(root: &OsStr, name: &OsStr, profile: &OsStr) -> Result<ExitCode, Box<dyn Error>> {
    let name = name.to_str().ok_or("contract must be valid UTF-8")?;
    let profile = profile.to_str().ok_or("profile must be valid UTF-8")?;
    let input = io::read_to_string(io::stdin().lock())?;
    let value = wire_json::parse(input.as_bytes())?;
    let registry = Registry::load(Path::new(root))?;
    match registry.validate(name, profile, &value) {
        Ok(value) => {
            let mut output = io::stdout().lock();
            serde_json::to_writer(&mut output, &value)?;
            writeln!(output)?;
            Ok(ExitCode::SUCCESS)
        }
        Err(ValidationError::Invalid(issue)) => {
            let mut output = io::stderr().lock();
            serde_json::to_writer(&mut output, &issue)?;
            writeln!(output)?;
            Ok(ExitCode::from(2))
        }
        Err(error) => Err(error.into()),
    }
}

fn run(args: &[OsString]) -> Result<ExitCode, Box<dyn Error>> {
    match args {
        [command, root, output, tools, openapi @ ..]
            if command == "generate-sdk" && openapi.len() <= 1 =>
        {
            generator::generate(
                Path::new(root),
                Path::new(output),
                Path::new(tools),
                openapi.first().map(Path::new),
            )?;
        }
        [command, root, contract, profile] if command == "validate" => {
            return validate(root, contract, profile);
        }
        [command] if command == "descriptor" => {
            io::stdout().lock().write_all(FILE_DESCRIPTOR_SET)?;
        }
        [command, namespace] if command == "descriptor" && namespace == "legacy" => {
            io::stdout().lock().write_all(LEGACY_FILE_DESCRIPTOR_SET)?;
        }
        [command] if command == "--help" || command == "-h" => {
            println!("{USAGE}");
        }
        _ => return Err(USAGE.into()),
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    run(&args).unwrap_or_else(|error| {
        eprintln!("{error}");
        ExitCode::FAILURE
    })
}
