extern crate core;
extern crate flow_impl;
extern crate flow_impl_derive;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate serde_json;

use flow_impl::{Implementation, RUN_AGAIN, RunAgain};
use flow_impl_derive::FlowImpl;
use serde_json::Value;

#[derive(FlowImpl)]
/// The struct for `ComposeArray` implementation
pub struct ComposeArray;

impl Implementation for ComposeArray {
    fn run(&self, mut inputs: Vec<Vec<Value>>) -> (Option<Value>, RunAgain) {
        let mut input_stream = inputs.remove(0);
        let mut output_vec = Vec::new();

        output_vec.push(input_stream.remove(0));
        output_vec.push(input_stream.remove(0));
        output_vec.push(input_stream.remove(0));
        output_vec.push(input_stream.remove(0));

        let output = Value::Array(output_vec);

        (Some(output), RUN_AGAIN)
    }
}