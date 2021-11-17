
pub mod aaa;
pub mod aas;
pub mod aap;
pub mod store;
pub mod interface;


use crate::{comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent}};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Receiver, Sender,};


pub struct ModuleStateCmd {
    pub cmd: &'static str,
    pub topic: String,
    pub data: Arc<Vec<u8>>,
}

lazy_static! {
    pub static ref CHANNEL_MODULE_STATE_CMD: (Mutex<Sender<ModuleStateCmd>>, Mutex<Receiver<ModuleStateCmd>>) = {
        let (sender, receiver) = std::sync::mpsc::channel::<ModuleStateCmd>();
        return (Mutex::new(sender), Mutex::new(receiver));
    };
}

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

    fn get_module(&self, key: &String) -> &MainboardConnectedModule {
        match self.connected_module.get(key.as_str()) {
            Some(m) => &m,
            None => panic!("Module not found for cnfig"),
        }
    }
}

fn get_module_validator(module_type: char, ) -> Box<dyn interface::ModuleValueValidator> {
    // TODO switch back to a match but i was having issue with match :(
    if module_type == 'A' {
        return Box::new(aaa::AAAValidator{});
    } else if module_type == 'S' {
        return Box::new(aas::AASValidator{});
    } else if module_type == 'P' {
        return Box::new(aap::AAPValidator{});
    } else {
        panic!("its a panic no validator found for type {}", module_type);
    }
}


fn send_module_state(
    id: &str,
    port: i32,
    state: bool,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
) -> () {
    let mut send_state = crate::protos::module::ModuleData::new();
    send_state.id = String::from(id);
    send_state.plug = state;
    send_state.atIndex = port;
    sender_socket.send((String::from(format!("/m/{}/state", id)), Box::new(send_state))).unwrap();
}

fn handle_module_state(
    manager: & mut MainboardModuleStateManager,
    state: & ModuleStateChangeEvent,
    sender_comboard_config: & Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: &store::ModuleStateStore,
) -> () {
    if !manager.connected_module.contains_key(state.id.as_str()) {
        if state.state == true {
            log::debug!("module connected {} at {}", state.id.as_str(), state.port);
            manager.connected_module.insert(state.id.clone(), MainboardConnectedModule{
                port: state.port,
                id: state.id.clone(),
            });
            send_module_state(state.id.as_str(), state.port, true, sender_socket);

            let config = store.get_module_config(&state.id);
            if config.is_some() {
                let t = state.id.chars().nth(2).unwrap();
                let validator = get_module_validator(t);

                // TODO implement fonction to handle not byte but structure directly
                let bytes = Arc::new(config.unwrap().write_to_bytes().unwrap());
                let (_config, config_comboard) = validator.apply_parse_config(state.port, t, bytes);
                sender_comboard_config.send(config_comboard).unwrap();
            } else {
                log::error!("cannot retrieve a config for {}", state.id);
            }
        } else {
            // receive state disconnect for a module a didnt know was connected
            log::error!("received disconnected event on not connected module {} at {}", state.id.as_str(), state.port)
        }
    } else {
        if state.state == false {
            log::debug!("Module disconnected {} at {}", state.id.as_str(), state.port);
            manager.connected_module.remove(state.id.as_str());
            send_module_state(state.id.as_str(), state.port, false, sender_socket);
        } else {
            // state is true but was already connected , weird
            log::error!("received connected event on already connected module {} at {}", state.id.as_str(), state.port);
        }
    }

}

fn handle_module_value(
    manager: & mut MainboardModuleStateManager,
    value: & ModuleValueValidationEvent,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
) -> () {

    let reference_connected_module = manager.get_module_at_index(value.port);
    log::debug!("got value for {}", reference_connected_module.id);

    let validator = get_module_validator(reference_connected_module.id.chars().nth(2).unwrap());
    
    let sensor_value = validator.convert_to_value(value);

    sender_socket
        .send((String::from(format!("/m/{}/data", reference_connected_module.id)), sensor_value))
        .expect("Failed to send !!!");
}

fn handle_sync_request(
    manager: & mut MainboardModuleStateManager,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
) -> () {
    log::debug!("send sync request to the cloud");
    for (k,v) in manager.connected_module.iter() {
        send_module_state(k, v.port, true, sender_socket);
    }
}

fn last_element_path(path: &str) -> String {
    let topic_elements: Vec<&str> = path.split("/").collect();
    return match topic_elements.len() {
        0 => panic!("Failed to get module id from path"),
        n => String::from(topic_elements[n - 1])
    };
}

fn handle_mconfig(
    manager: & mut MainboardModuleStateManager,
    store: & store::ModuleStateStore,
    sender_comboard_config: & Sender<crate::comboard::imple::interface::Module_Config>,
    id: String,
    data: Arc<Vec<u8>>
) -> () {
    let module_ref = manager.get_module(&id);
    let t = module_ref.id.chars().nth(2).unwrap();
    let validator = get_module_validator(t);

    let (config, config_comboard) = validator.apply_parse_config(module_ref.port, t, data);

    store.store_module_config(&id, config);
    sender_comboard_config.send(config_comboard).unwrap();
}

pub async fn module_state_task(
    receiver_state_change: Receiver<ModuleStateChangeEvent>,
    receiver_value_validation: Receiver<ModuleValueValidationEvent>,
    sender_comboard_config: Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: store::ModuleStateStore,
) {
    let mut manager = MainboardModuleStateManager{
        connected_module: HashMap::new(),
    };

    loop {
        {
            let receive = receiver_state_change.try_recv();
            if receive.is_ok() {
                let state = receive.unwrap();
                handle_module_state(& mut manager, &state, &sender_comboard_config, &sender_socket, &store);
            }
        }
        {
            let receive = receiver_value_validation.try_recv();
            if receive.is_ok() {
                let value = receive.unwrap();
                handle_module_value(& mut manager, &value, &sender_socket);
            }
        }
        {
            let receive = CHANNEL_MODULE_STATE_CMD.1.lock().unwrap().try_recv();
            if receive.is_ok() {
                let cmd = receive.unwrap();
                match cmd.cmd {
                    "sync" => handle_sync_request(& mut manager, &sender_socket),
                    "mconfig" => handle_mconfig(& mut manager, &store, &sender_comboard_config, last_element_path(&cmd.topic), cmd.data),
                    _ => log::error!("receive invalid cmd {}", cmd.cmd),
                }
            }
        }
    }

}
