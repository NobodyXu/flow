use flow_impl::{Implementation, RUN_AGAIN, RunAgain};
use flow_impl_derive::FlowImpl;
use serde_json::Value;

#[derive(FlowImpl)]
/// Convert an input type to a String
///
/// ## Include using
/// ```toml
/// [[process]]
/// alias = "to_string"
/// source = "lib://flowstdlib/fmt/to_string"
/// ```
///
/// ## Input
/// * The data to convert to a String. Current types supported are:
/// * String - a bit redundant, but it works
/// * Bool - Boolean JSON value
/// * Number - A JSON Number
/// * Array - An JSON array of values that can be converted, they are converted one by one
///
/// ## Output
/// * The String equivalent of the input value
#[derive(Debug)]
pub struct ToString;

impl Implementation for ToString {
    fn run(&self, mut inputs: Vec<Vec<Value>>) -> (Option<Value>, RunAgain) {
        let mut value = None;

        let input = inputs.remove(0).remove(0);
        match input {
            Value::String(_) => {
                value = Some(input);
            },
            Value::Bool(boolean) => {
                let val = Value::String(boolean.to_string());
                value = Some(val);
            },
            Value::Number(number) => {
                let val = Value::String(number.to_string());
                value = Some(val);
            },
            _ => {}
        };

        (value, RUN_AGAIN)
    }
}