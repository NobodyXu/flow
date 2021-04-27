use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use simpath::{FileType, FoundType, Simpath};

use lib_path::check_flow_lib_path;

mod lib_path;

// Build script to compile the flowstdlib WASM files and generate manifest - using flowc
fn main() -> io::Result<()> {
    let flowc = get_flowc()?;

    let mut command = Command::new(flowc);
    // Options for flowc: -g for debug symbols, -z to dump graphs, -l for a library build
    let command_args = vec!["-v", "info", "-g", "-z", "-l", env!("CARGO_MANIFEST_DIR")];

    command
        .args(command_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    check_flow_lib_path();

    Ok(())
}

fn get_flowc() -> io::Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let dev = root.join("target/debug/flowc");
    if dev.exists() {
        return Ok(dev);
    }

    let dev = root.join("target/release/flowc");
    if dev.exists() {
        return Ok(dev);
    }

    if let Ok(FoundType::File(flowc)) = Simpath::new("PATH").find_type("flowc", FileType::File) {
        return Ok(flowc);
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "`flowc` could not be found in `$PATH` or `target/`",
    ))
}
