use std::sync::mpsc::Sender;

use crate::{modulestate::interface::ModuleValueParsable, mainboardstate::error::MainboardError};



pub type SenderPayload = (String, Box<dyn ModuleValueParsable>);


pub struct SenderSocket {
    sender_socket: Sender<SenderPayload>,
}


impl SenderSocket {
    pub fn send(&self, topic: String, value: Box<dyn ModuleValueParsable>) -> Result<(), MainboardError> {
        self.sender_socket.send((topic, value))?;
        Ok(())
    }
}


impl From<Sender<SenderPayload>> for SenderSocket {
    fn from(value: Sender<SenderPayload>) -> Self {
        Self {
            sender_socket: value
        }
    }

}
