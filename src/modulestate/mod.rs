

pub mod aap;
pub mod interface;

use crate::{comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent}, protos::message::GrowbeMessage};
use std::collections::HashMap;

struct MainboardConnectedModule {
    pub port: i32,
    pub id: String,
}

struct MainboardModuleStateManager {
    pub connected_module: HashMap<String, MainboardConnectedModule>,
}


impl MainboardModuleStateManager {
    fn getModuleAtIndex(_port: i32) -> () {

    }
}

pub async fn moduleStateTask(
    receiverStateChange: std::sync::mpsc::Receiver<ModuleStateChangeEvent>,
    receiverValueValidation: std::sync::mpsc::Receiver<ModuleValueValidationEvent>,

    senderSocket: std::sync::mpsc::Sender<GrowbeMessage>,
) {
    let mut manager = MainboardModuleStateManager{
        connected_module: HashMap::new(),
    };

    loop {
        {
            let receive = receiverStateChange.try_recv();
            if receive.is_ok() {
                let state = receive.unwrap();

                // a new module not connected is connected
                if !manager.connected_module.contains_key(state.id.as_str()) {
                    if state.state == true {
                        manager.connected_module.insert(state.id.clone(), MainboardConnectedModule{
                            port: state.port,
                            id: state.id.clone(),
                        });
                    } else {
                        // receive state disconnect for a module a didnt know was connected
                    }
                } else {
                    if state.state == false {
                        manager.connected_module.remove(state.id.as_str());
                    } else {
                        // state is true but was already connected , weird
                    }
                }
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
