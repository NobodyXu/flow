use flowclib::compiler::loader;
use provider::content::provider::MetaProvider;

#[path="helper.rs"] mod helper;

#[test]
fn load_hello_world_from_context() {
    helper::set_flow_lib_path();
    let meta_provider = MetaProvider {};
    loader::load_context(&helper::absolute_file_url_from_relative_path("samples/hello-world/context.toml"),
                         &meta_provider).unwrap();
}

#[test]
fn load_hello_world_include() {
    helper::set_flow_lib_path();
    let meta_provider = MetaProvider {};
    loader::load_context(&helper::absolute_file_url_from_relative_path("samples/hello-world-include/context.toml"),
                         &meta_provider).unwrap();
}

#[test]
fn load_hello_world_flow1() {
    helper::set_flow_lib_path();
    let meta_provider = MetaProvider {};
    loader::load_context(&helper::absolute_file_url_from_relative_path("samples/hello-world/flow1.toml"),
                         &meta_provider).unwrap();
}

#[test]
fn load_reverse_echo_from_toml() {
    helper::set_flow_lib_path();
    let meta_provider = MetaProvider {};
    loader::load_context(&helper::absolute_file_url_from_relative_path("samples/reverse-echo/context.toml"),
                         &meta_provider).unwrap();
}

#[test]
fn load_fibonacci_from_file() {
    helper::set_flow_lib_path();
    let meta_provider = MetaProvider {};
    loader::load_context(&helper::absolute_file_url_from_relative_path("samples/fibonacci/context.toml"),
                         &meta_provider).unwrap();
}

#[test]
fn load_fibonacci_from_directory() {
    helper::set_flow_lib_path();
    let meta_provider = MetaProvider {};
    let url = helper::absolute_file_url_from_relative_path("samples/fibonacci");
    loader::load_context(&url, &meta_provider).unwrap();
}