use std::{ffi::OsString, fmt::format};

use tokio::sync::mpsc::error::{TrySendError, SendError};

use crate::modulestate::interface::ModuleError;


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

    pub fn unauthorized(msg: &str) -> Self {
        return Self {
            message: format!("unauthorized: {}", msg)
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

impl From<tokio_tungstenite::tungstenite::Error> for MainboardError {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        return Self {
            message: value.to_string(),
        }
    }
}

//tokio::sync::mpsc::error::SendError<(std::string::String, Box<(dyn ModuleValueParsable + 'static)>)>>>
impl <T> From<TrySendError<T>> for MainboardError {
    fn from(value: TrySendError<T>) -> Self {
        return Self {
            message: value.to_string(),
        };
    }
}

impl <T> From<SendError<T>> for MainboardError {
    fn from(value: SendError<T>) -> Self {
        return Self {
            message: value.to_string(),
        };
    }
}

impl From<nix::errno::Errno> for MainboardError {
    fn from(value: nix::errno::Errno) -> Self {
        return Self {
            message: value.to_string(),
        }
    }
}

impl From<OsString> for MainboardError {
    fn from(_value: OsString) -> Self {
        return Self {
            message: "failed to cast os string".into(),
        }
    }
}
