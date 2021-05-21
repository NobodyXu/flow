#[macro_use]
extern crate error_chain;

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

use simpath::Simpath;
use url::Url;

use flowclib::compiler::{compile, loader};
use flowclib::generator::generate;
use flowclib::generator::generate::GenerationTables;
use flowclib::model::flow::Flow;
use flowclib::model::process::Process;
use flowclib::model::process::Process::FlowProcess;
use flowcore::lib_provider::MetaProvider;

#[path = "helper.rs"]
mod helper;

#[doc(hidden)]
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

#[doc(hidden)]
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
    flow: &Flow,
    debug_symbols: bool,
    out_dir: PathBuf,
    test_name: &str,
    tables: &GenerationTables,
) -> Result<PathBuf> {
    let mut filename = out_dir;
    filename.push(&format!("{}.json", test_name));
    let mut manifest_file =
        File::create(&filename).chain_err(|| "Could not create manifest file")?;
    let out_dir_path =
        Url::from_file_path(&filename).map_err(|_| "Could not create filename url")?;

    let manifest = generate::create_manifest(
        &flow,
        debug_symbols,
        &out_dir_path,
        tables,
        HashSet::<(Url, Url)>::new(),
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

fn execute_flow(filepath: PathBuf, test_args: Vec<String>, input: String) -> (String, String) {
    let mut command = Command::new("cargo");
    let mut command_args = vec![
        "run",
        "--quiet",
        "-p",
        "flowr",
        "--",
        "-n",
        filepath.to_str().unwrap(),
    ];
    for test_arg in &test_args {
        command_args.push(test_arg);
    }

    let mut child = command
        .args(command_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // send it stdin from the "${testname}.stdin" file
    write!(child.stdin.unwrap(), "{}", input).unwrap();

    // read stdout
    let mut output = String::new();
    if let Some(ref mut stdout) = child.stdout {
        for line in BufReader::new(stdout).lines() {
            output.push_str(&format!("{}\n", &line.unwrap()));
        }
    }

    // read stderr
    let mut err = String::new();
    if let Some(ref mut stderr) = child.stderr {
        for line in BufReader::new(stderr).lines() {
            err.push_str(&format!("{}\n", &line.unwrap()));
        }
    }

    (output, err)
}

fn test_args(test_dir: &Path, test_name: &str) -> Vec<String> {
    let test_args = format!("{}.args", test_name);
    let mut args_file = test_dir.to_path_buf();
    args_file.push(test_args);
    let f = File::open(&args_file).unwrap();
    let f = BufReader::new(f);

    let mut args = Vec::new();
    for line in f.lines() {
        args.push(line.unwrap());
    }
    args
}

fn load_flow(test_dir: &Path, test_name: &str, search_path: Simpath) -> Process {
    let test_flow = format!("{}.toml", test_name);
    let mut flow_file = test_dir.to_path_buf();
    flow_file.push(test_flow);
    loader::load(
        &helper::absolute_file_url_from_relative_path(&flow_file.to_string_lossy()),
        &MetaProvider::new(search_path),
        &mut HashSet::<(Url, Url)>::new(),
    )
    .unwrap()
}

fn get(test_dir: &Path, file_name: &str) -> String {
    let mut expected_file = test_dir.to_path_buf();
    expected_file.push(file_name);
    let mut f = File::open(&expected_file).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

fn execute_test(test_name: &str, search_path: Simpath) {
    // helper::set_lib_search_path()
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();
    let test_dir = root_dir.join(&format!("flowc/tests/test-flows/{}", test_name));

    if let FlowProcess(ref flow) = load_flow(&test_dir, test_name, search_path) {
        let tables = compile::compile(flow).unwrap();
        let out_dir = test_dir.clone();
        let manifest_path = write_manifest(flow, true, out_dir, test_name, &tables).unwrap();

        let test_args = test_args(&test_dir, test_name);
        let input = get(&test_dir, &format!("{}.stdin", test_name));
        let (actual_stdout, actual_stderr) = execute_flow(manifest_path, test_args, input);
        let expected_output = get(&test_dir, &format!("{}.expected", test_name));
        assert_eq!(
            expected_output, actual_stdout,
            "Flow output did not match that in .expected file"
        );
        assert!(
            actual_stderr.is_empty(),
            "There was stderr output during test: \n{}",
            actual_stderr
        )
    }
}

#[test]
#[test]
fn print_args() {
    let search_path = helper::set_lib_search_path_to_project();
    execute_test("print-args", search_path);
}

#[test]
#[test]
fn hello_world() {
    let search_path = helper::set_lib_search_path_to_project();
    execute_test("hello-world", search_path);
}

#[test]
#[test]
fn line_echo() {
    let search_path = helper::set_lib_search_path_to_project();
    execute_test("line-echo", search_path);
}

#[test]
#[test]
fn args() {
    let search_path = helper::set_lib_search_path_to_project();
    execute_test("args", search_path);
}

#[test]
#[test]
fn args_json() {
    let search_path = helper::set_lib_search_path_to_project();
    execute_test("args_json", search_path);
}

#[test]
#[test]
fn array_input() {
    let search_path = helper::set_lib_search_path_to_project();
    execute_test("array-input", search_path);
}
