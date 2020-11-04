use std::sync::{Arc, Mutex};

use serde_json::Value;

use flow_impl::{DONT_RUN_AGAIN, Implementation, RUN_AGAIN, RunAgain};

use crate::client_server::RuntimeServerConnection;
use crate::runtime::{Event, Response};

/// `Implementation` struct for the `Stdin` function
pub struct Stdin {
    /// It holds a reference to the runtime client in order to read input
    pub server_context: Arc<Mutex<RuntimeServerConnection>>
}

impl Implementation for Stdin {
    fn run(&self, _inputs: &[Value]) -> (Option<Value>, RunAgain) {
        if let Ok(mut server) = self.server_context.lock() {
            return match server.send_event(Event::GetStdin) {
                Ok(Response::Stdin(contents)) => (Some(Value::String(contents)), RUN_AGAIN),
                Ok(Response::GetStdinEOF) => (Some(Value::Null), DONT_RUN_AGAIN),
                _ => (None, DONT_RUN_AGAIN)
            }
        }
        (None, DONT_RUN_AGAIN)
    }
}