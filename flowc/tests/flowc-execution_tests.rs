#[macro_use]
extern crate error_chain;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::fmt::Write as FormatWrite;

use serial_test::serial;
use simpath::Simpath;
use tempdir::TempDir;
use url::Url;

use flowclib::compiler::{compile, loader};
use flowclib::compiler::compile::CompilerTables;
use flowclib::generator::generate;
use flowcore::meta_provider::MetaProvider;
use flowcore::model::flow_definition::FlowDefinition;
use flowcore::model::process::Process;
use flowcore::model::process::Process::FlowProcess;

#[path = "helper.rs"]
mod helper;

#[doc(hidden)]
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

error_chain! {
    foreign_links {
        Provider(::flowcore::errors::Error);
        Compiler(::flowclib::errors::Error);
        Io(::std::io::Error);
    }
}

/// Execution tests
///
/// These are a set of System tests that compile a sample flow, and then execute it, capturing
/// the output and comparing it to the expected output.

fn write_manifest(
    flow: &FlowDefinition,
    debug_symbols: bool,
    out_dir: PathBuf,
    test_name: &str,
    tables: &CompilerTables,
) -> Result<PathBuf> {
    let mut filename = out_dir;
    filename.push(&format!("{}.json", test_name));
    let mut manifest_file =
        File::create(&filename).chain_err(|| "Could not create manifest file")?;
    let out_dir_path =
        Url::from_file_path(&filename).map_err(|_| "Could not create filename url")?;

    let manifest = generate::create_manifest(
        flow,
        debug_symbols,
        &out_dir_path,
        tables,
        &HashSet::<(Url, Url)>::new(),
    )?;

    manifest_file
        .write_all(
            serde_json::to_string_pretty(&manifest)
                .chain_err(|| "Could not pretty format json for manifest")?
                .as_bytes(),
        )
        .chain_err(|| "Could not writ manifest data bytes to file")?;

    Ok(filename)
}

fn execute_flow(
    filepath: PathBuf,
    test_args: Vec<String>,
    input: String,
    separate_processes: bool,
) -> (String, String) {
    let context_root = helper::get_context_root();
    let context_root_str = context_root.to_str().expect("Could not get context root");
    let server = if separate_processes {
        println!("Starting the 'flowr' server");
        let mut server_command = Command::new("cargo");
        // flowr args: -n for native libs, -s to get a server process
        let server_command_args = vec!["run", "--quiet", "-p", "flowr", "--", "-n", "-s",
        "-C", context_root_str];

        // spawn the 'flowr' server child process
        Some(
            server_command
                .args(server_command_args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to spawn flowc as server"),
        )
    } else {
        None
    };

    let mut command = Command::new("cargo");
    let mut command_args = vec!["run", "--quiet", "-p", "flowr", "--"];

    if separate_processes {
        // start another 'flowr' process in client mode
        command_args.push("-c");
    } else {
        // when running client_and_server in same process we want to use native libs
        command_args.push("-n");
    }

    // Append the file path to the manifest to run to the command line args
    command_args.push(filepath.to_str().expect("Could not convert path to string"));

    // Append the flow arguments to the end of the command line
    for test_arg in &test_args {
        command_args.push(test_arg);
    }

    // spawn the 'flowr' child process
    let mut runner = command
        .args(command_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Could not spawn flowr child process");

    // send it stdin from the "${testname}.stdin" file
    write!(runner.stdin.expect("Could not get stdin"), "{}", input)
        .expect("Could not write to stdin");

    // read it's stdout
    let mut output = String::new();
    if let Some(ref mut stdout) = runner.stdout {
        for line in BufReader::new(stdout).lines() {
            let _ = writeln!(output, "{}", &line.expect("Could not read line"));
        }
    }

    // read it's stderr
    let mut err = String::new();
    if let Some(ref mut stderr) = runner.stderr {
        for line in BufReader::new(stderr).lines() {
            let _ = writeln!(err, "{}", &line.expect("Could not read line"));
        }
    }

    // If a server was started - then kill it
    if let Some(mut server_child) = server {
        println!("Killing 'flowr' server");
        server_child.kill().expect("Failed to kill server child process");
    }

    (output, err)
}

fn test_args(test_dir: &Path, test_name: &str) -> Vec<String> {
    let test_args = format!("{}.args", test_name);
    let mut args_file = test_dir.to_path_buf();
    args_file.push(test_args);
    let f = File::open(&args_file).expect("Could not open args file for test");
    let f = BufReader::new(f);

    let mut args = Vec::new();
    for line in f.lines() {
        args.push(line.expect("Could not get line to append to"));
    }
    args
}

fn load_flow(test_dir: &Path, test_name: &str, search_path: Simpath) -> Process {
    let test_flow = format!("{}.toml", test_name);
    let mut flow_file = test_dir.to_path_buf();
    flow_file.push(test_flow);
    loader::load(
        &helper::absolute_file_url_from_relative_path(&flow_file.to_string_lossy()),
        &MetaProvider::new(search_path, helper::get_context_root()),
        &mut HashSet::<(Url, Url)>::new(),
    )
    .expect("Could not load process")
}

fn get(test_dir: &Path, file_name: &str) -> String {
    let mut expected_file = test_dir.to_path_buf();
    expected_file.push(file_name);
    let mut f = File::open(&expected_file).expect("Could not open file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Could not read from file");
    String::from_utf8(buffer).expect("Could not convert to String")
}

fn execute_test(test_name: &str, separate_processes: bool) {
    let search_path = helper::set_lib_search_path_to_project();
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();
    let test_dir = root_dir.join(&format!("flowc/tests/test-flows/{}", test_name));

    if let FlowProcess(ref flow) = load_flow(&test_dir, test_name, search_path) {
        let mut source_urls = HashSet::<(Url, Url)>::new();
        let output_dir = TempDir::new("flow-test").expect("A temp dir").into_path();

        let tables = compile::compile(flow,
                                          &output_dir, false,  false,
                                      #[cfg(feature = "debugger")] &mut source_urls
        ).expect("Could not compile flow");
        let dir =
            TempDir::new("flow").expect("Could not get temp dir");
        let manifest_path = write_manifest(flow, true, dir.into_path(), test_name, &tables)
            .expect("Could not write manifest file");
        let test_args = test_args(&test_dir, test_name);
        let input = get(&test_dir, &format!("{}.stdin", test_name));
        let (actual_stdout, actual_stderr) =
            execute_flow(manifest_path, test_args, input, separate_processes);
        let expected_output = get(&test_dir, &format!("{}.expected", test_name));
        assert!(actual_stderr.is_empty(), "{}", actual_stderr);
        assert_eq!(
            expected_output, actual_stdout,
            "Flow output did not match that in .expected file"
        );
    }
}

#[cfg(feature = "debugger")]
#[test]
#[serial]
fn debug_print_args() {
    execute_test("debug-print-args", false);
}

#[test]
#[serial]
fn print_args() {
    execute_test("print-args", false);
}

#[test]
#[serial]
fn hello_world() {
    execute_test("hello-world", false);
}

#[test]
#[serial]
fn line_echo() {
    execute_test("line-echo", false);
}

#[test]
#[serial]
fn args() {
    execute_test("args", false);
}

#[test]
#[serial]
fn args_json() {
    execute_test("args_json", false);
}

#[test]
#[serial]
fn array_input() {
    execute_test("array-input", false);
}

#[test]
#[serial]
fn double_connection() {
    execute_test("double-connection", false);
}

#[test]
#[serial]
fn duplicate_connection() {
    execute_test("duplicate-connection", false);
}

#[test]
#[serial]
fn two_destinations() {
    execute_test("two_destinations", false);
}

#[ignore]
#[test]
#[serial]
fn hello_world_client_server() {
    execute_test("hello-world", true);
}
