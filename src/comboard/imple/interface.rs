use std::sync::mpsc::Receiver;

pub static I2C_VIRT_ID: &'static str = "virt";

pub struct ModuleStateChangeEvent {
    pub board: String,
    pub board_addr: String,
    pub port: i32,
    pub id: String,
    pub state: bool,
}

pub struct ModuleValueValidationEvent {
    pub board: String,
    pub board_addr: String,
    pub port: i32,
    pub buffer: Vec<u8>,
}

pub struct ComboardClientConfig {
    pub config: String,
}

pub trait ComboardClient {
    fn run(
        &self,
        receiver_config: Receiver<super::channel::ModuleConfig>,
    ) -> tokio::task::JoinHandle<Result<(), ()>>;
}
