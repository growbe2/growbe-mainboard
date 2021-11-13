use std::sync::{Mutex, Arc, mpsc::Sender, mpsc::Receiver};

#[repr(C)]
pub struct Module_Config {
    pub port: cty::c_int,
    pub buffer: [cty::uint8_t; 512],
}

pub struct ModuleStateChangeEvent {
    pub port: i32,
    pub id: String,
    pub state: bool,
}

pub struct ModuleValueValidationEvent {
    pub port: i32,
    pub buffer: Vec<u8>,
}

pub struct ComboardClientConfig {
    pub receiverConfig: Arc<Mutex<Receiver<Module_Config>>>,
    pub senderStateChange: Arc<Mutex<Sender<ModuleStateChangeEvent>>>,
    pub senderValueValidation: Arc<Mutex<Sender<ModuleValueValidationEvent>>>,
}

pub trait ComboardClient {
	fn run(&self, config: ComboardClientConfig) -> tokio::task::JoinHandle<()>;
}
