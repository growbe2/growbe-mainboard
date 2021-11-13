

pub mod aap;
pub mod interface;

use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};
use std::collections::HashMap;

struct MainboardConnectedModule {
    pub port: i32,
    pub id: &'static str,
}

struct MainboardModuleStateManager {
    pub connectedModule: HashMap<String, MainboardConnectedModule>,
}


impl MainboardModuleStateManager {
    fn getModuleAtIndex(_port: i32) -> () {

    }
}

pub async fn moduleStateTask(
    receiverStateChange: std::sync::mpsc::Receiver<ModuleStateChangeEvent>,
    receiverValueValidation: std::sync::mpsc::Receiver<ModuleValueValidationEvent>,
) {
    let _manager = MainboardModuleStateManager{
        connectedModule: HashMap::new(),
    };

    loop {
        {
            let receive = receiverStateChange.try_recv();
            if receive.is_ok() {
                let state = receive.unwrap();
                println!("Receive a state youpi {}", state.id);
            }
        }
        {
            let receive = receiverValueValidation.try_recv();
            if receive.is_ok() {
                let value = receive.unwrap();
                println!("Receive value my dear {}", value.buffer[3])
            }
        }
    }

}
