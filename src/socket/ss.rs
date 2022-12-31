use tokio::sync::mpsc::Sender;

use crate::{mainboardstate::error::MainboardError, modulestate::interface::ModuleValueParsable};

#[derive(Debug)]
pub enum SenderPayloadData {
    ProtobufMessage(Box<dyn protobuf::Message>),
    Buffer(Vec<u8>),
}

pub type SenderPayload = (String, SenderPayloadData);

pub struct SenderSocket {
    sender_socket: Sender<SenderPayload>,
}

impl Clone for SenderSocket {
    fn clone(&self) -> Self {
        Self {
            sender_socket: self.sender_socket.clone(),
        }
    }
}

impl SenderSocket {
    pub fn send(
        &self,
        topic: String,
        value: Box<dyn ModuleValueParsable>,
    ) -> Result<(), MainboardError> {
        self.sender_socket
            .try_send((topic, SenderPayloadData::ProtobufMessage(value)))?;
        Ok(())
    }
}

impl From<Sender<SenderPayload>> for SenderSocket {
    fn from(value: Sender<SenderPayload>) -> Self {
        Self {
            sender_socket: value,
        }
    }
}
