//! Build script to compile the flow flowsamples in the crate
use std::{fs, io};
use std::path::Path;
use std::process::{Command, Stdio};

#[allow(clippy::collapsible_if)]
fn main() -> io::Result<()> {
    let samples_root = env!("CARGO_MANIFEST_DIR");

    println!("cargo:rerun-if-env-changed=FLOW_LIB_PATH");
    // Tell Cargo that if any file in the flowsamples directory changes it should rerun this build script
    println!("cargo:rerun-if-changed={}", samples_root);

    println!("`flowsample` version {}", env!("CARGO_PKG_VERSION"));
    println!(
        "Current Working Directory: `{}`",
        std::env::current_dir()
            .expect("Could not read the Current Working Directory")
            .display()
    );
    println!("Samples Root Directory: `{}`", env!("CARGO_MANIFEST_DIR"));

    // find all sample sub-folders
    for entry in fs::read_dir(samples_root)? {
        let e = entry?;
        if e.file_type()?.is_dir() {
            println!("Building sample '{}'", e.path().to_str().expect("Could not convert path to string"));
            if let Err(err) = compile_sample(&e.path()) {
                eprintln!("Sample build failed with message:\n{}", err);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn compile_sample(sample_dir: &Path) -> io::Result<()> {
    let mut command = Command::new("flowc");
    // -g for debug symbols, -z to dump graphs, -v warn to show warnings, -s to skip running and only compile the flow
    let command_args = vec!["-g", "-z", "-v", "warn", "-s", sample_dir.to_str().expect("Could not get directory as string")];

    let flowc_command = command
        .args(&command_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let flowc_output = flowc_command.output()?;

    match flowc_output.status.code() {
        Some(0) | None => {}
        Some(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Error building sample, command line\n {}{}",
                "flowc ", command_args.join(" "))))
        }
    }

    Ok(())
}
