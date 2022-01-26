#[cfg(test)]
pub mod test {
    use std::sync::{Arc, Mutex};

    use crate::client_server::{ClientConnection, ServerConnection, WAIT};
    use crate::coordinator::RUNTIME_SERVICE_NAME;
    use crate::runtime_messages::{ClientMessage, ServerMessage};

    pub fn wait_for_then_send(
        wait_for_message: ServerMessage,
        then_send: ClientMessage,
    ) -> Arc<Mutex<ServerConnection>> {
        let server_connection = Arc::new(Mutex::new(
            ServerConnection::new("tcp", RUNTIME_SERVICE_NAME, None)
                .expect("Could not create server connection"),
        ));

        let connection = server_connection.lock()
            .expect("Could not get access to server connection");
        let mut server_info = connection.get_server_info().clone();
        let client_connection = ClientConnection::new(&mut server_info)
            .expect("Could not create ClientConnection");

        client_connection
            .send(ClientMessage::Ack)
            .expect("Could not send initial 'Ack' message");

        std::thread::spawn(move || loop {
            match client_connection.receive::<ServerMessage>() {
                Ok(received_message) => {
                    if received_message == wait_for_message {
                        client_connection
                            .send(then_send)
                            .expect("Could not send ClientMessage");

                        return;
                    }
                }
                _ => panic!("Error receiving ServerMessage"),
            }
        });

        // Get the initial Ack sent from client to open the connection
        let guard = server_connection
            .lock()
            .expect("Could not get a lock on the server connection");
        guard
            .receive::<ClientMessage>(WAIT)
            .expect("Could not receive initial Ack message from client");

        server_connection.clone()
    }
}
