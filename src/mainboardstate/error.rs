use tokio::sync::mpsc::error::TrySendError;

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

//tokio::sync::mpsc::error::SendError<(std::string::String, Box<(dyn ModuleValueParsable + 'static)>)>>>
impl From<TrySendError<(String, Box<dyn ModuleValueParsable + 'static>)>> for MainboardError {
    fn from(value: TrySendError<(String, Box<dyn ModuleValueParsable>)>) -> Self {
        return Self {
            message: value.to_string(),
        };
    }
}
