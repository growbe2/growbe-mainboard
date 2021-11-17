use std::sync::{Mutex, Arc, mpsc::Sender, mpsc::Receiver};

#[repr(C)]
pub struct Module_Config {
    pub port: cty::c_int,
    pub buffer: [cty::uint8_t; 8],
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
    pub receiver_config: Arc<Mutex<Receiver<Module_Config>>>,
    pub sender_state_change: Arc<Mutex<Sender<ModuleStateChangeEvent>>>,
    pub sender_value_validation: Arc<Mutex<Sender<ModuleValueValidationEvent>>>,
}

pub trait ComboardClient {
	fn run(&self, config: ComboardClientConfig) -> tokio::task::JoinHandle<()>;
}
