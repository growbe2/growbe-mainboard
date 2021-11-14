

pub mod aap;
pub mod aas;
pub mod interface;

use crate::{comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent}, protos::message::GrowbeMessage};
use crate::protos::module::{SOILModuleData};
use std::collections::HashMap;

use protobuf::Message;

struct MainboardConnectedModule {
    pub port: i32,
    pub id: String,
}

struct MainboardModuleStateManager {
    pub connected_module: HashMap<String, MainboardConnectedModule>,
}


impl MainboardModuleStateManager {
    fn get_module_at_index(&self, port: i32) -> &MainboardConnectedModule {
        for (_, v) in self.connected_module.iter() {
            if v.port == port {
                return &v;
            }
        }
        panic!("NOT FOUND");
    }

}


fn get_module_validator<T: protobuf::Message + interface::ModuleValue>(module_type: char, ) -> Box<dyn interface::ModuleValueValidator<T>> where aap::AAPValidator: interface::ModuleValueValidator<T> {
    return match module_type {
        'P' => Box::new(aap::AAPValidator{}),
        //'S' => Box::new(aas::AASValidator{}),
        _ => panic!("Panico pratique"),
    }
}


fn handle_module_state(manager: & mut MainboardModuleStateManager, state: & ModuleStateChangeEvent) -> () {
    if !manager.connected_module.contains_key(state.id.as_str()) {
        if state.state == true {
            println!("Module connected {} at {}", state.id.as_str(), state.port);
            manager.connected_module.insert(state.id.clone(), MainboardConnectedModule{
                port: state.port,
                id: state.id.clone(),
            });
        } else {
            // receive state disconnect for a module a didnt know was connected
            println!("Received disconnected event on not connected module {} at {}", state.id.as_str(), state.port)
        }
    } else {
        if state.state == false {
            println!("Module disconnected {} at {}", state.id.as_str(), state.port);
            manager.connected_module.remove(state.id.as_str());
        } else {
            // state is true but was already connected , weird
            println!("Received connected event on already connected module {} at {}", state.id.as_str(), state.port);
        }
    }

}

fn handle_module_value(manager: & mut MainboardModuleStateManager, value: & ModuleValueValidationEvent) -> () {

    let reference_connected_module = manager.get_module_at_index(value.port);
    println!("Got value for {}", reference_connected_module.id);
    let validator = get_module_validator(reference_connected_module.id.chars().nth(2).unwrap());
    
    let sensor_value = validator.convert_to_value(value);

}

pub async fn moduleStateTask(
    receiverStateChange: std::sync::mpsc::Receiver<ModuleStateChangeEvent>,
    receiverValueValidation: std::sync::mpsc::Receiver<ModuleValueValidationEvent>,
    //senderSocket: std::sync::mpsc::Sender<GrowbeMessage>,
) {
    let mut manager = MainboardModuleStateManager{
        connected_module: HashMap::new(),
    };

    loop {
        {
            let receive = receiverStateChange.try_recv();
            if receive.is_ok() {
                let state = receive.unwrap();
                handle_module_state(& mut manager, &state);
            }
        }
        {
            let receive = receiverValueValidation.try_recv();
            if receive.is_ok() {
                let value = receive.unwrap();
                handle_module_value(& mut manager, &value);
            }
        }
    }

}
