use std::env;
use std::path::Path;

use simpath::Simpath;
use url::Url;

pub fn set_lib_search_path() -> Simpath {
    let mut lib_search_path = Simpath::new("lib_search_path");
    let root_str = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("Could not get project root dir");
    lib_search_path.add_directory(root_str.to_str().unwrap());

    let runtime_parent = root_str.join("flowr/src/lib");
    lib_search_path.add_directory(runtime_parent.to_str().unwrap());

    println!("Lib search path set to '{}'", lib_search_path);

    lib_search_path
}

pub fn absolute_file_url_from_relative_path(path: &str) -> String {
    let flow_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    Url::from_directory_path(flow_root).unwrap().join(path).unwrap().to_string()
}