use tokio::sync::mpsc::{Receiver, Sender};

use serde::{Deserialize, Serialize};

use crate::channel::{ModuleValue, ModuleConfig};

use self::{ccs::StreamingModule, css::SystemStatsModule};

pub mod ccs;
pub mod css;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Module {
    pub name: String,
    pub port: i32,
}

pub trait ModuleClient {
    fn run(
        &self,
        receiver_config: Receiver<ModuleConfig>,
        sender_value: Sender<ModuleValue>,
    ) -> tokio::task::JoinHandle<Result<(), ()>>;
}

pub fn get_module_client(name: &str) -> Option<Box<dyn ModuleClient>> {
    match name {
        "CCS" => Some(Box::new(StreamingModule::new())),
        "CSS" => Some(Box::new(SystemStatsModule::new())),
        _ => None,
    }
}
