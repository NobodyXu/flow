use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use url::Url;

use flowclib::compiler::loader;
use flowclib::deserializers::deserializer_helper::get_deserializer;
use flowclib::model::name::HasName;
use flowclib::model::process::Process::FunctionProcess;
use flowrlib::lib_manifest::DEFAULT_LIB_MANIFEST_FILENAME;
use flowrlib::lib_manifest::LibraryManifest;
use flowrlib::manifest::MetaData;
use flowrlib::provider::Provider;
use glob::glob;

use crate::compile_wasm;
use crate::errors::*;

/*
    Compile a Library
*/
pub fn build_lib(url: Url, skip_building: bool, lib_dir: PathBuf, provider: &dyn Provider) -> Result<String> {
    let library = loader::load_library(&url.to_string(), provider)
        .expect(&format!("Could not load Library from '{}'", lib_dir.display()));

    info!("Building manifest for '{}' in output directory: '{}'\n", library.name, lib_dir.display());
    let mut lib_manifest = LibraryManifest::new(MetaData::from(&library));

    build_manifest(&mut lib_manifest, &lib_dir.to_str().unwrap(), provider, skip_building)
        .expect("Could not build library");

    let filename = write_lib_manifest(&lib_manifest, lib_dir)?;
    info!("Generated library manifest at '{}'", filename.display());

    Ok("ok".into())
}

/*
    Generate a manifest for the library in JSON that can be used to load it using 'flowr'
*/
fn write_lib_manifest(lib_manifest: &LibraryManifest, base_dir: PathBuf) -> Result<PathBuf> {
    let mut filename = base_dir.clone();
    filename.push(DEFAULT_LIB_MANIFEST_FILENAME.to_string());
    let mut manifest_file = File::create(&filename).chain_err(|| "Could not create lib manifest file")?;

    manifest_file.write_all(serde_json::to_string_pretty(lib_manifest)
        .chain_err(|| "Could not pretty format the library manifest JSON contents")?
        .as_bytes()).chain_err(|| "Could not write library smanifest data bytes to created manifest file")?;

    Ok(filename)
}

fn build_manifest(lib_manifest: &mut LibraryManifest, base_dir: &str, provider: &dyn Provider,
                  skip_building: bool) -> Result<()> {
    let search_pattern = if base_dir.ends_with("/") {
        format!("{}**/*.toml", base_dir)
    } else {
        format!("{}/**/*.toml", base_dir)
    };

    debug!("Searching for process definitions using search pattern: '{}':\n", search_pattern);
    for entry in glob(&search_pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(ref toml_path) => {
                let resolved_url = Url::from_file_path(&toml_path)
                    .map_err(|_| format!("Could not create url from file path '{}'",
                                         toml_path.to_str().unwrap()))?.to_string();
                debug!("Inspecting '{}' for function definition", resolved_url);
                let contents = provider.get(&resolved_url)
                    .chain_err(|| format!("Could not get contents of resolved url: '{}'", resolved_url))?;
                let deserializer = get_deserializer(&resolved_url)?;
                match deserializer.deserialize(&String::from_utf8(contents).unwrap(), Some(&resolved_url)) {
                    Ok(FunctionProcess(ref mut function)) => {
                        function.set_implementation_url(&resolved_url);
                        let wasm_abs_path = compile_wasm::compile_implementation(function, skip_building)?;
                        let wasm_dir = wasm_abs_path.parent().expect("Could not get parent directory of wasm path");
                        lib_manifest.add_to_manifest(base_dir,
                                                     wasm_abs_path.to_str().expect("Could not convert wasm_path to str"),
                                                     wasm_dir.to_str().expect("Could not convert wasm_dir to str"),
                                                     function.name() as &str);
                    }
                    _ => { /* Ignore errors and valid flow definitions */ }
                }
            }
            Err(_) => { /* Skipping unreadable files */ }
        }
    }

    Ok(())
}