use std::sync::{Arc, Mutex};

use flow_impl::{Implementation, RUN_AGAIN, RunAgain};
use serde_json::Value;

use flowrlib::runtime_client::{Command, Response, RuntimeClient};

/// `Implementation` struct for the `image_buffer` function
#[derive(Debug)]
pub struct ImageBuffer {
    /// It holds a reference to the runtime client in order to send commands
    pub client: Arc<Mutex<dyn RuntimeClient>>
}

impl Implementation for ImageBuffer {
    fn run(&self, inputs: &[Value]) -> (Option<Value>, RunAgain) {
        let pixel = inputs[0].as_array().unwrap();
        let value = inputs[1].as_array().unwrap();
        let size = inputs[2].as_array().unwrap();
        let filename = inputs[3].to_string();

        if let Ok(mut client) = self.client.lock() {
            match client.send_command(Command::PixelWrite(
                (pixel[0].as_u64().unwrap() as u32, pixel[1].as_u64().unwrap() as u32),
                (value[0].as_u64().unwrap() as u8, value[1].as_u64().unwrap() as u8, value[2].as_u64().unwrap() as u8),
                (size[0].as_u64().unwrap() as u32, size[1].as_u64().unwrap() as u32),
                filename
            )) {
                Response::Ack => return (None, RUN_AGAIN),
                _ => return (None, RUN_AGAIN)
            }
        }

        (None, RUN_AGAIN)
    }
}