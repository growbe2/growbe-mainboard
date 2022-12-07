use std::sync::mpsc::SendError;

use crate::modulestate::interface::ModuleError;
use crate::modulestate::interface::ModuleValueParsable;

#[derive(Debug, Clone)]
pub struct MainboardError {
    pub message: String,
}

impl MainboardError {
    // CONSTRUCTOR
    pub fn new() -> Self {
        return Self {
            message: String::from(""),
        };
    }

    pub fn not_found(typ: &str, id: &str) -> Self {
        return Self {
            message: format!("{} with id {} not found", typ, id),
        };
    }

    pub fn from_send_error<T>(err: SendError<T>) -> Self {
        log::error!("{:?}", err);
        return Self {
            message: "failed to send payload to mpsc sender".to_string(),
        };
    }

    pub fn from_sqlite_err(err: rusqlite::Error) -> Self {
        return Self {
            message: err.to_string(),
        };
    }

    pub fn from_protobuf_err(err: protobuf::ProtobufError) -> Self {
        return Self {
            message: err.to_string(),
        };
    }

    // BUILDER METHODS

    pub fn from_error(err: String) -> Self {
        return Self { message: err };
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }
}

impl From<ModuleError> for MainboardError {
    fn from(value: ModuleError) -> Self {
        return Self {
            message: value.to_string(),
        };
    }
}

impl From<rusqlite::Error> for MainboardError {
    fn from(value: rusqlite::Error) -> Self {
        return Self {
            message: value.to_string(),
        };
    }
}

impl From<protobuf::ProtobufError> for MainboardError {
    fn from(value: protobuf::ProtobufError) -> Self {
        return Self {
            message: value.to_string(),
        };
    }
}

impl From<SendError<(String, Box<dyn ModuleValueParsable>)>> for MainboardError {
    fn from(value: SendError<(String, Box<dyn ModuleValueParsable>)>) -> Self {
        return Self {
            message: value.to_string(),
        };
    }
}
