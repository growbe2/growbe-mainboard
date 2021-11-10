#[repr(C)]
pub struct Module_Config {
    pub port: cty::c_int,
    pub buffer: [cty::uint8_t; 512],
}

pub struct ModuleStateChangeEvent {

}

pub struct ModuleValueValidationEvent {

}

pub struct ComboardClientConfig {
    pub receiverConfig: std::sync::mpsc::Receiver<Module_Config>,
    pub senderStateChange: std::sync::mpsc::Sender<ModuleStateChangeEvent>,
    pub senderValueValidation: std::sync::mpsc::Sender<ModuleValueValidationEvent>,
}

pub trait ComboardClient {
	fn run(&self,config: ComboardClientConfig) -> std::thread::JoinHandle<()>;
}
