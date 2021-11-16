
pub mod aaa;
pub mod aas;
pub mod interface;

use crate::{comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent}};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Receiver, Sender,};


pub struct ModuleStateCmd {
    pub cmd: &'static str,
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
}

fn get_module_validator(module_type: char, ) -> Box<dyn interface::ModuleValueValidator> {
    // TODO switch back to a match but i was having issue with match :(
    if module_type == 'A' {
        return Box::new(aaa::AAAValidator{});
    } else if module_type == 'S' {
        return Box::new(aas::AASValidator{});
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
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    conn: &Mutex<rusqlite::Connection>,
) -> () {
    if !manager.connected_module.contains_key(state.id.as_str()) {
        if state.state == true {
            println!("Module connected {} at {}", state.id.as_str(), state.port);
            manager.connected_module.insert(state.id.clone(), MainboardConnectedModule{
                port: state.port,
                id: state.id.clone(),
            });
            send_module_state(state.id.as_str(), state.port, true, sender_socket);
        } else {
            // receive state disconnect for a module a didnt know was connected
            println!("Received disconnected event on not connected module {} at {}", state.id.as_str(), state.port)
        }
    } else {
        if state.state == false {
            println!("Module disconnected {} at {}", state.id.as_str(), state.port);
            manager.connected_module.remove(state.id.as_str());
            send_module_state(state.id.as_str(), state.port, false, sender_socket);
        } else {
            // state is true but was already connected , weird
            println!("Received connected event on already connected module {} at {}", state.id.as_str(), state.port);
        }
    }

}

fn handle_module_value(
    manager: & mut MainboardModuleStateManager,
    value: & ModuleValueValidationEvent,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
) -> () {

    let reference_connected_module = manager.get_module_at_index(value.port);
    println!("Got value for {}", reference_connected_module.id);

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
    println!("Send sync request to the cloud");
    for (k,v) in manager.connected_module.iter() {
        send_module_state(k, v.port, false, sender_socket);
    }
}

pub async fn module_state_task(
    receiver_state_change: Receiver<ModuleStateChangeEvent>,
    receiver_value_validation: Receiver<ModuleValueValidationEvent>,
    sender_socket: Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    conn: Arc<Mutex<rusqlite::Connection>>,
) {
    let mut manager = MainboardModuleStateManager{
        connected_module: HashMap::new(),
    };

    loop {
        {
            let receive = receiver_state_change.try_recv();
            if receive.is_ok() {
                let state = receive.unwrap();
                handle_module_state(& mut manager, &state, &sender_socket, &conn);
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
                    _ => println!("MODULE_STATE receive invalid cmd"),
                }
            }
        }
    }

}
