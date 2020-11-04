use std::sync::{Arc, Mutex};

use serde_json::Value;

use flow_impl::{Implementation, RUN_AGAIN, RunAgain};

use crate::client_server::RuntimeServerConnection;
use crate::runtime::{Event, Response};

/// `Implementation` struct for the `file_write` function
pub struct FileWrite {
    /// It holds a reference to the runtime client in order to get file contents
    pub server_context: Arc<Mutex<RuntimeServerConnection>>
}

impl Implementation for FileWrite {
    fn run(&self, inputs: &[Value]) -> (Option<Value>, RunAgain) {
        let filename = &inputs[0];
        let bytes = &inputs[1];

        if let Ok(mut server) = self.server_context.lock() {
            return match server.send_event(Event::Write(filename.to_string(),
                                                        bytes.as_str().unwrap().as_bytes().to_vec())) {
                Ok(Response::Ack) => (None, RUN_AGAIN),
                _ => (None, RUN_AGAIN)
            }
        }

        (None, RUN_AGAIN)
    }
}