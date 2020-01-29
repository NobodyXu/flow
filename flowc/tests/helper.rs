use std::env;
use std::path::PathBuf;

use url::Url;

pub fn url_relative_to_flow_route(path: &str) -> String {
    let mut flow_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    flow_root.pop();
    println!("manifest_location = {:?}", flow_root);
    let abs_url = Url::from_directory_path(flow_root).unwrap().join(path).unwrap().to_string();
    println!("Absoluete URL = {}", abs_url);
    abs_url
}

pub fn set_flow_lib_path() {
    let mut flow_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    flow_root.pop();
    println!("Set 'FLOW_LIB_PATH' to '{}'", flow_root.to_string_lossy().to_string());
    env::set_var("FLOW_LIB_PATH", flow_root.to_string_lossy().to_string());
}