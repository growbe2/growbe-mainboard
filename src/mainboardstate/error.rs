use std::sync::mpsc::SendError;

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

    pub fn from_send_error<T>(err: SendError<T>) -> Self {
        log::error!("{:?}", err);
        return Self {
            message: "failed to send payload to mpsc sender".to_string(),
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
