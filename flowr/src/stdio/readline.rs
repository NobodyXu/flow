use std::io::{self};

use flowrlib::implementation::DONT_RUN_AGAIN;
use flowrlib::implementation::Implementation;
use flowrlib::implementation::RunAgain;
use serde_json::Value as JsonValue;

pub struct Readline;

impl Implementation for Readline {
    fn run(&self, _inputs: Vec<Vec<JsonValue>>) -> (Option<JsonValue>, RunAgain) {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n > 0 {
                    let value = JsonValue::String(input.trim().to_string());
                    return (Some(value), true);
                }
            }
            Err(_) => {}
        }

        (None, DONT_RUN_AGAIN)
    }
}