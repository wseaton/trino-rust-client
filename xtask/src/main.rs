// xtask/src/lib.rs
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, fs, path::PathBuf, process::Stdio};

use trino_codegen::explain::ExplainRoot;
use trino_codegen::utils::generate_struct;

pub fn test_generated_struct() -> Result<(), Box<dyn std::error::Error>> {
    // Generate the struct and write it to a file

    let root: ExplainRoot =
        serde_json::from_str(include_str!("../../trino-codegen/data/explain2.json")).unwrap();
    let data = root.outputs;

    let path = Path::new("src/generated_struct.rs");
    let mut file = File::create(path)?;

    let imports = vec![
        "use serde::Deserialize;\n",
        "use trino_codegen::utils::dates;\n",
        "use trino_codegen::utils::binary;\n",
        "use trino_codegen::utils::array;\n",
    ];

    for import in &imports {
        file.write_all(import.as_bytes()).unwrap();
    }

    let generated_struct = generate_struct(&data, path);

    let _ = file.write(generated_struct.to_string().as_bytes());

    // Compile the code
    let status = Command::new("cargo").arg("build").status()?;
    if !status.success() {
        return Err("Failed to compile the code".into());
    }

    let status = Command::new("cargo").arg("fmt").status()?;
    if !status.success() {
        return Err("Failed to format the code".into());
    }

    // Run the test
    let status = Command::new("cargo")
        .args(&["test", "--test", "test_generated_struct"])
        .status()?;
    if !status.success() {
        return Err("Test failed".into());
    }

    Ok(())
}

type DynError = Box<dyn std::error::Error>;
fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("test_generated_struct") => test_generated_struct()?,
        _ => print_help(),
    }
    Ok(())
}
fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn print_help() {
    eprintln!(
        "Tasks:

        test_generated_struct            test the generated struct
"
    )
}
