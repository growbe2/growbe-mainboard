
pub mod aaa;
pub mod aas;
pub mod aap;
pub mod store;
pub mod relay;
pub mod aab;
pub mod interface;
pub mod alarm;


use crate::{comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent}};
use crate::comboard::imple::channel::*;
use crate::protos::alarm::FieldAlarm;
use lazy_static::lazy_static;
use protobuf::Message;
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
    pub handler_map: std::collections::HashMap<i32, tokio_util::sync::CancellationToken>,
    pub last_value: Option<Box<dyn interface::ModuleValueParsable>>,
}

struct MainboardModuleStateManager {
    pub connected_module: HashMap<String, MainboardConnectedModule>,
}


impl MainboardModuleStateManager {
    fn get_module_at_index(&self, port: i32) -> Option<&MainboardConnectedModule> {
        for (_, v) in self.connected_module.iter() {
            if v.port == port {
                return Some(&v);
            }
        }
        return None;
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
    } else if module_type == 'B' {
        return Box::new(aab::AABValidator{});
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
    // error ici parfois
    sender_socket.send((String::from(format!("/m/{}/state", id)), Box::new(send_state))).unwrap();
}

#[inline]
fn handle_module_state(
    manager: & mut MainboardModuleStateManager,
    state: & ModuleStateChangeEvent,
    sender_comboard_config: & Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: &store::ModuleStateStore,
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
) -> () {
    if state.state == true {
            log::debug!("module connected {} at {}", state.id.as_str(), state.port);
            manager.connected_module.insert(state.id.clone(), MainboardConnectedModule{
                port: state.port,
                id: state.id.clone(),
                handler_map: std::collections::HashMap::new(),
                last_value: None,
            });
            send_module_state(state.id.as_str(), state.port, true, sender_socket);

            let config = store.get_module_config(&state.id);
            if config.is_some() {
                let t = state.id.chars().nth(2).unwrap();
                let validator = get_module_validator(t);

                // TODO implement fonction to handle not byte but structure directly
                let module_mut_ref = manager.connected_module.get_mut(state.id.as_str()).unwrap();
                let bytes = Arc::new(config.unwrap().write_to_bytes().unwrap());
                match validator.apply_parse_config(state.port, t, bytes, sender_comboard_config, &mut module_mut_ref.handler_map) {
                    Ok((_config, config_comboard)) => sender_comboard_config.send(config_comboard).unwrap(),
                    Err(e) => log::error!("{}", e),
                }
                tokio::task::spawn(async {});
            } else {
                log::warn!("cannot retrieve a config for {}", state.id);
            }

            let alarms_result = alarm_store.get_alarm_for_module(&state.id.clone());
            if let Ok(mut alarms) = alarms_result {
                log::info!("loading {} alarms for {}", alarms.len(), state.id.as_str());
                for _n in 0..alarms.len() {
                    let alarm = alarms.pop().unwrap();
                    alarm_validator.register_field_alarm(alarm).unwrap();
                }
            }
    }
    if state.state == false {
        log::debug!("Module disconnected {} at {}", state.id.as_str(), state.port);
        // remove from module map
        let connected_module = manager.connected_module.remove(state.id.as_str());
        // clear task  
        if connected_module.is_some() {
            connected_module.unwrap().handler_map.iter().for_each(|module| module.1.cancel());
            send_module_state(state.id.as_str(), state.port, false, sender_socket);
        }
        // clear alarm
        let alarms_result = alarm_store.get_alarm_for_module(&state.id.clone());
        if let Ok(mut alarms) = alarms_result {
            log::info!("removing {} alarms for {}", alarms.len(), state.id.as_str());
            for _n in 0..alarms.len() {
                let alarm = alarms.pop().unwrap();
                alarm_validator.deregister_field_alarm(alarm).unwrap();
            }
        }
   }
}

fn handle_module_value(
    manager: & mut MainboardModuleStateManager,
    value: & ModuleValueValidationEvent,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
) -> () {

    let reference_connected_module_option = manager.get_module_at_index(value.port);
    if reference_connected_module_option.is_none() {
        log::error!("receive value for port {} but module is not in the store", value.port);
        return;
    }
    let reference_connected_module = reference_connected_module_option.unwrap();

    let validator = get_module_validator(reference_connected_module.id.chars().nth(2).unwrap());


    let on_change = |value| {
       sender_socket
                .send((String::from(format!("/m/{}/data", reference_connected_module.id)), value))
                .expect("Failed to send !!!");

    };
    
    match validator.convert_to_value(value) {

        Ok(sensor_value) => {
            if reference_connected_module.last_value.is_some() {
                let change = validator.have_data_change(&sensor_value, reference_connected_module.last_value.as_ref().unwrap());
                if change.0 == true {
                    on_change(sensor_value);
                    if let Ok(previous_value) = validator.convert_to_value(value) {
                        let module_ref = manager.connected_module.get_mut(reference_connected_module.id.clone().as_str()).unwrap();
                        module_ref.last_value = Some(previous_value);

                        log::debug!("data have changed for {}", module_ref.id.as_str());

                        if change.1.len() > 0 {
                            let module_value_change = alarm::model::ModuleValueChange::<i32>{
                                module_id: module_ref.id.clone(),
                                changes: change.1
                            };
                            alarm_validator.on_module_value_change(&module_value_change).iter()
                                .map(|event| Box::new(event))
                                .for_each(|event| sender_socket.send((format!("/m/{}/alarm", event.moduleId), Box::new(event.clone_me()))).unwrap());
                        }
                    }
                }
            } else {
                on_change(sensor_value);
                if let Ok(previous_value) = validator.convert_to_value(value) {
                    let module_ref = manager.connected_module.get_mut(reference_connected_module.id.clone().as_str()).unwrap();
                    module_ref.last_value = Some(previous_value);
                }
            }
        },
        Err(e) => log::error!("{}", e),
    }
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


fn handle_add_alarm(
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
    data: Arc<Vec<u8>>,
) -> () {
    let field_alarm = FieldAlarm::parse_from_bytes(&data).unwrap();
    alarm_store.add_alarm_field(&field_alarm).unwrap();
    alarm_validator.register_field_alarm(field_alarm).unwrap();
}

fn handle_remove_alarm(
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
    data: Arc<Vec<u8>>,
) -> () {
    let field_alarm = FieldAlarm::parse_from_bytes(&data).unwrap();
    alarm_store.remove_alarm_field(&field_alarm).unwrap();
    alarm_validator.deregister_field_alarm(field_alarm).unwrap();
}

pub fn module_state_task(
    sender_socket: Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: store::ModuleStateStore,
    alarm_store: alarm::store::ModuleAlarmStore,
) -> tokio::task::JoinHandle<()> {
    let mut manager = MainboardModuleStateManager{
        connected_module: HashMap::new(),
    };
    
    let sender_config = CHANNEL_CONFIG.0.lock().unwrap().clone();
        

    return tokio::spawn(async move {
        let mut alarm_validator = alarm::validator::AlarmFieldValidator::new();

        let receiver_state = CHANNEL_STATE.1.lock().unwrap();
        let receiver_value = CHANNEL_VALUE.1.lock().unwrap();


        loop {
            {
                let receive = receiver_state.try_recv();
                if receive.is_ok() {
                    let state = receive.unwrap();
                    handle_module_state(& mut manager, &state, &sender_config, &sender_socket, &store, & mut alarm_validator,&alarm_store);
                }
            }
            {
                let receive = receiver_value.try_recv();
                if receive.is_ok() {
                    let value = receive.unwrap();
                    handle_module_value(& mut manager, &value, &sender_socket, &mut alarm_validator);
                }
            }
            {
                let receive = CHANNEL_MODULE_STATE_CMD.1.lock().unwrap().try_recv();
                if receive.is_ok() {
                    let cmd = receive.unwrap();
                    match cmd.cmd {
                        "sync" => handle_sync_request(& mut manager, &sender_socket),
                        "mconfig" => {
                                let id = last_element_path(&cmd.topic);
                                let module_ref_option = manager.connected_module.get_mut(id.as_str());

                                if let Some(module_ref) = module_ref_option {

                                        let t = module_ref.id.chars().nth(2).unwrap();
                                        let validator = get_module_validator(t);

                                        match validator.apply_parse_config(module_ref.port, t, cmd.data, &sender_config, &mut module_ref.handler_map) {
                                            Ok((config, config_comboard)) => {
                                                store.store_module_config(&id, config);
                                                sender_config.send(config_comboard).unwrap();
                                            },
                                            Err(e) => log::error!("{}", e)
                                        }
                                        tokio::task::spawn(async {});
                                } else {
                                    log::error!("Receive config for unplug module not supported {}", id.as_str());
                        }
                            //handle_mconfig(& mut manager, &store,  last_element_path(&cmd.topic), cmd.data).await
                    },
                        "aAl" => handle_add_alarm(& mut alarm_validator, &alarm_store, cmd.data),
                        "rAl" => handle_remove_alarm(& mut alarm_validator, &alarm_store, cmd.data),
                        _ => log::error!("receive invalid cmd {}", cmd.cmd),
                    }
                }
            }
        }
    });


}
