use std::collections::HashMap;
use std::collections::HashSet;

use error_chain::bail;
use log::info;
use serde_derive::Serialize;

use flowrlib::function::Function as RuntimeFunction;
use flowrlib::input::Input;
use flowrlib::manifest::{Manifest, MetaData};

use crate::errors::*;
use crate::model::connection::Connection;
use crate::model::flow::Flow;
use crate::model::function::Function;
use crate::model::io::IO;
use crate::model::library::Library;
use crate::model::name::HasName;
use crate::model::route::HasRoute;
use crate::model::route::Route;

#[derive(Serialize, Default)]
pub struct GenerationTables {
    pub connections: Vec<Connection>,
    pub source_routes: HashMap<Route, (Route, usize)>,
    /// HashMap from "route of the output of a function" --> (output name, source_function_id)
    pub destination_routes: HashMap<Route, (usize, usize, usize)>,
    /// HashMap from "route of the input of a function" --> (dest_function_id, input number, flow_id)
    pub collapsed_connections: Vec<Connection>,
    pub functions: Vec<Function>,
    pub libs: HashSet<String>,
}

impl GenerationTables {
    pub fn new() -> Self {
        GenerationTables {
            connections: Vec::new(),
            source_routes: HashMap::<Route, (Route, usize)>::new(),
            destination_routes: HashMap::<Route, (usize, usize, usize)>::new(),
            collapsed_connections: Vec::new(),
            functions: Vec::new(),
            libs: HashSet::new()
        }
    }
}

impl From<&Flow> for MetaData {
    fn from(flow: &Flow) -> Self {
        MetaData {
            name: flow.name.clone().to_string(),
            description: flow.description.clone(),
            version: flow.version.clone(),
            author_name: flow.author_name.clone(),
            author_email: flow.author_email.clone(),
        }
    }
}

impl From<&Library> for MetaData {
    fn from(library: &Library) -> Self {
        MetaData {
            name: library.name.clone().to_string(),
            description: library.description.clone(),
            version: library.version.clone(),
            author_name: library.author_name.clone(),
            author_email: library.author_email.clone(),
        }
    }
}

impl From<&IO> for Input {
    fn from(io: &IO) -> Self {
        Input::new(io.depth(),
                   io.get_initializer())
    }
}

/*
    Paths in the manifest are relative to the location of the manifest file, to make the file
    and associated files relocatable (and manybe packagable into a ZIP etc). So we use manifest_dir
    as the root directory other file paths are made relatiove to.
*/
pub fn create_manifest(flow: &Flow, debug_symbols: bool, manifest_dir: &str, tables: &GenerationTables)
                       -> Result<Manifest> {
    info!("Writing flow manifest to '{}'", manifest_dir);

    let mut manifest = Manifest::new(MetaData::from(flow));

    // Generate run-time Process struct for each of the functions
    for function in &tables.functions {
        manifest.add_function(function_to_runtimefunction(&manifest_dir, function, debug_symbols)?);
    }

    manifest.lib_references = tables.libs.clone();

    Ok(manifest)
}

/*
    Create a run-time function struct from a compile-time function struct.
    manifest_dir is the directory that paths will be made relative to.
*/
fn function_to_runtimefunction(manifest_dir: &str, function: &Function, debug_symbols: bool) -> Result<RuntimeFunction> {
    let name = if debug_symbols {
        function.alias().to_string()
    } else { "".to_string() };

    let route = if debug_symbols {
        function.route().to_string()
    } else { "".to_string() };

    // make the location of implementation relative to the output directory if it is under it
    let implementation_location = implementation_location_relative(&function, manifest_dir)?;

    let mut runtime_inputs = vec!();
    match &function.get_inputs() {
        &None => {}
        Some(inputs) => {
            for input in inputs {
                runtime_inputs.push(Input::from(input));
            }
        }
    };

    Ok(RuntimeFunction::new(name,
                            route,
                            implementation_location,
                            runtime_inputs,
                            function.get_id(), function.get_flow_id(),
                            function.get_output_routes(),
                            debug_symbols))
}

/*
    Get the location of the implementation - relative to the Manifest if it is a provided implementation
*/
fn implementation_location_relative(function: &Function, out_dir: &str) -> Result<String> {
    if let Some(ref lib_reference) = function.get_lib_reference() {
        Ok(format!("lib://{}/{}", lib_reference, &function.name()))
    } else {
        match &function.get_implementation() {
            Some(implementation_path) => {
                info!("Out_dir = '{}'", out_dir);
                info!("Absolute implementation path = '{}'", implementation_path);
                let relative_path = implementation_path.replace(out_dir, "");
                info!("Absolute implementation path = '{}'", relative_path);
                Ok(relative_path)
            }
            None => {
                bail!("Function '{}' is not a lib reference but no implementation is provided", function.name())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use flowrlib::input::{ConstantInputInitializer, InputInitializer};
    use flowrlib::input::OneTimeInputInitializer;
    use flowrlib::output_connection::OutputConnection;

    use crate::model::function::Function;
    use crate::model::io::IO;
    use crate::model::name::Name;
    use crate::model::route::Route;

    use super::function_to_runtimefunction;

    #[test]
    fn function_with_sub_route_output_generation() {
        let function = Function::new(
            Name::from("Stdout"),
            false,
            Some("lib://flowruntime/stdio/stdout".to_string()),
            Name::from("print"),
            Some(vec!()),
            Some(vec!(
                IO::new("Value", &Route::default()),
                IO::new("String", &Route::default())
            )),
            "file:///fake/file",
            Route::from("/flow0/stdout"),
            None,
            vec!(OutputConnection::new("".to_string(), 1, 0, 0, 0, false, None),
                 OutputConnection::new("sub_route".to_string(), 2, 0, 0, 0, false, None)),
            0, 0);

        let expected = "{
  'id': 0,
  'flow_id': 0,
  'implementation_location': 'lib://flowruntime/stdio/stdout',
  'output_routes': [
    {
      'function_id': 1,
      'io_number': 0,
      'flow_id': 0
    },
    {
      'subpath': 'sub_route',
      'function_id': 2,
      'io_number': 0,
      'flow_id': 0
    }
  ]
}";

        let br = Box::new(function) as Box<Function>;

        let runtime_process = function_to_runtimefunction("/test", &br, false).unwrap();

        let serialized_process = serde_json::to_string_pretty(&runtime_process).unwrap();
        assert_eq!(serialized_process, expected.replace("'", "\""));
    }

    #[test]
    fn function_generation() {
        let function = Function::new(
            Name::from("Stdout"),
            false,
            Some("lib://flowruntime/stdio/stdout".to_string()),
            Name::from("print"),
            Some(vec!()),
            Some(vec!(IO::new("String", &Route::default()))),
            "file:///fake/file",
            Route::from("/flow0/stdout"),
            None,
            vec!(OutputConnection::new("".to_string(), 1, 0, 0, 0, false, None)),
            0, 0);

        let expected = "{
  'id': 0,
  'flow_id': 0,
  'implementation_location': 'lib://flowruntime/stdio/stdout',
  'output_routes': [
    {
      'function_id': 1,
      'io_number': 0,
      'flow_id': 0
    }
  ]
}";

        let br = Box::new(function) as Box<Function>;

        let process = function_to_runtimefunction("/test", &br, false).unwrap();

        let serialized_process = serde_json::to_string_pretty(&process).unwrap();
        assert_eq!(serialized_process, expected.replace("'", "\""));
    }

    #[test]
    fn function_generation_with_array_order() {
        let function = Function::new(
            Name::from("Stdout"),
            false,
            Some("lib://flowruntime/stdio/stdout".to_string()),
            Name::from("print"),
            Some(vec!()),
            Some(vec!(IO::new("String", &Route::default()))),
            "file:///fake/file",
            Route::from("/flow0/stdout"),
            None,
            vec!(OutputConnection::new("".to_string(), 1, 0, 0,
                                       1, false, None)),
            0, 0);

        let expected = "{
  'id': 0,
  'flow_id': 0,
  'implementation_location': 'lib://flowruntime/stdio/stdout',
  'output_routes': [
    {
      'function_id': 1,
      'io_number': 0,
      'flow_id': 0,
      'array_order': 1
    }
  ]
}";

        let br = Box::new(function) as Box<Function>;

        let process = function_to_runtimefunction("/test", &br, false).unwrap();

        let serialized_process = serde_json::to_string_pretty(&process).unwrap();
        assert_eq!(serialized_process, expected.replace("'", "\""));
    }

    #[test]
    fn function_with_initialized_input_generation() {
        let mut io = IO::new("String", &Route::default());
        io.set_initializer(&Some(InputInitializer::OneTime(
            OneTimeInputInitializer { once: json!(1) }
        )));

        let function = Function::new(
            Name::from("Stdout"),
            false,
            Some("lib://flowruntime/stdio/stdout".to_string()),
            Name::from("print"),
            Some(vec!(io)),
            None,
            "file:///fake/file",
            Route::from("/flow0/stdout"),
            None,
            vec!(),
            0, 0);

        let expected = "{
  'id': 0,
  'flow_id': 0,
  'implementation_location': 'lib://flowruntime/stdio/stdout',
  'inputs': [
    {
      'initializer': {
        'once': 1
      }
    }
  ]
}";

        let br = Box::new(function) as Box<Function>;
        let process = function_to_runtimefunction("/test", &br, false).unwrap();

        println!("process {}", process);

        let serialized_process = serde_json::to_string_pretty(&process).unwrap();
        assert_eq!(expected.replace("'", "\""), serialized_process);
    }

    #[test]
    fn function_with_constant_input_generation() {
        let mut io = IO::new("String", &Route::default());
        io.set_initializer(&Some(InputInitializer::Constant(
            ConstantInputInitializer { constant: json!(1) }
        )));

        let function = Function::new(
            Name::from("Stdout"),
            false,
            Some("lib://flowruntime/stdio/stdout".to_string()),
            Name::from("print"),
            Some(vec!(io)),
            None,
            "file:///fake/file",
            Route::from("/flow0/stdout"),
            None,
            vec!(),
            0, 0);

        let expected = "{
  'id': 0,
  'flow_id': 0,
  'implementation_location': 'lib://flowruntime/stdio/stdout',
  'inputs': [
    {
      'initializer': {
        'constant': 1
      }
    }
  ]
}";

        let br = Box::new(function) as Box<Function>;
        let process = function_to_runtimefunction("/test", &br, false).unwrap();

        println!("process {}", process);

        let serialized_process = serde_json::to_string_pretty(&process).unwrap();
        assert_eq!(expected.replace("'", "\""), serialized_process);
    }

    #[test]
    fn function_with_array_input_generation() {
        let io = IO::new("Array/String", &Route::default());

        let function = Function::new(
            Name::from("Stdout"),
            false,
            Some("lib://flowruntime/stdio/stdout".to_string()),
            Name::from("print"),
            Some(vec!(io)),
            None,
            "file:///fake/file",
            Route::from("/flow0/stdout"),
            None,
            vec!(),
            0, 0);

        let expected = "{
  'id': 0,
  'flow_id': 0,
  'implementation_location': 'lib://flowruntime/stdio/stdout',
  'inputs': [
    {}
  ]
}";

        let br = Box::new(function) as Box<Function>;
        let process = function_to_runtimefunction("/test", &br, false).unwrap();

        println!("process {}", process);

        let serialized_process = serde_json::to_string_pretty(&process).unwrap();
        assert_eq!(serialized_process, expected.replace("'", "\""));
    }

    fn test_function() -> Function {
        Function::new(
            Name::from("Stdout"),
            false,
            Some("lib://flowruntime/stdio/stdout".to_string()),
            Name::from("print"),
            Some(vec!()),
            Some(vec!(
                IO::new("String", &Route::default())
            )),
            "file:///fake/file",
            Route::from("/flow0/stdout"),
            None,
            vec!(OutputConnection::new("".to_string(), 1, 0, 0, 0, false, None)),
            0, 0)
    }

    #[test]
    fn function_to_code_with_debug_generation() {
        let function = test_function();

        let expected = "{
  'name': 'print',
  'route': '/flow0/stdout',
  'id': 0,
  'flow_id': 0,
  'implementation_location': 'lib://flowruntime/stdio/stdout',
  'output_routes': [
    {
      'function_id': 1,
      'io_number': 0,
      'flow_id': 0
    }
  ]
}";

        let br = Box::new(function) as Box<Function>;

        let process = function_to_runtimefunction("/test", &br, true).unwrap();

        let serialized_process = serde_json::to_string_pretty(&process).unwrap();
        assert_eq!(serialized_process, expected.replace("'", "\""));
    }

    #[test]
    fn function_with_array_element_output_generation() {
        let function = Function::new(
            Name::from("Stdout"),
            false,
            Some("lib://flowruntime/stdio/stdout".to_string()),
            Name::from("print"),
            Some(vec!()),
            Some(vec!(IO::new("Array", &Route::default()))),
            "file:///fake/file",
            Route::from("/flow0/stdout"),
            None,
            vec!(OutputConnection::new("/0".to_string(), 1, 0, 0, 0, false, None)),
            0, 0);

        let expected = "{
  'id': 0,
  'flow_id': 0,
  'implementation_location': 'lib://flowruntime/stdio/stdout',
  'output_routes': [
    {
      'subpath': '/0',
      'function_id': 1,
      'io_number': 0,
      'flow_id': 0
    }
  ]
}";

        let br = Box::new(function) as Box<Function>;

        let process = function_to_runtimefunction("/test", &br, false).unwrap();

        let serialized_process = serde_json::to_string_pretty(&process).unwrap();
        assert_eq!(serialized_process, expected.replace("'", "\""));
    }
}