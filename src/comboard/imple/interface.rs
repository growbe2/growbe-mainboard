
use tokio::sync::mpsc::{Receiver, Sender};

use crate::modulestate::interface::ModuleMsg;

pub static I2C_VIRT_ID: &'static str = "virt";

#[derive(Debug)]
pub struct ModuleStateChangeEvent {
    pub board: String,
    pub board_addr: String,
    pub port: i32,
    pub id: String,
    pub state: bool,
}

#[derive(Debug)]
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
        &mut self,
        sender_module: Sender<ModuleMsg>,
        receiver_config: Receiver<super::channel::ModuleConfig>,
    ) -> tokio::task::JoinHandle<Result<(), ()>>;
}
