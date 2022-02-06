
pub mod aaa;
pub mod aas;
pub mod aap;
pub mod store;
pub mod relay;
pub mod aab;
pub mod interface;
pub mod alarm;
pub mod actor;


use crate::{comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent}};
use crate::comboard::imple::channel::*;
use crate::protos::alarm::FieldAlarm;
use interface::ModuleStateCmd;
use lazy_static::lazy_static;
use protobuf::Message;
use std::{collections::HashMap};
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Receiver, Sender,};
use aab::AABValidator;

use self::relay::virtual_relay::handler::on_module_state_changed_virtual_relays;

lazy_static! {
    pub static ref CHANNEL_MODULE_STATE_CMD: (Mutex<Sender<ModuleStateCmd>>, Mutex<Receiver<ModuleStateCmd>>) = {
        let (sender, receiver) = std::sync::mpsc::channel::<ModuleStateCmd>();
        return (Mutex::new(sender), Mutex::new(receiver));
    };
}


pub struct MainboardConnectedModule {
    pub port: i32,
    pub id: String,
    pub handler_map: std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
    pub last_value: Option<Box<dyn interface::ModuleValueParsable>>,
    pub validator: Box<dyn interface::ModuleValueValidator>,
}

pub struct MainboardModuleStateManager {
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
    // cheap hack plz can i do better
    fn get_module_at_index_mut(&mut self, port: i32) -> Option<&mut MainboardConnectedModule> {
        let mut id: String = String::from("");
        {
            if let Some(its_a_me_variable) = self.get_module_at_index(port) {
                id = its_a_me_variable.id.clone();
            }
        }
        return self.connected_module.get_mut(&id);
    }

    fn get_connected_modules(&self) -> Vec<String> {
        return Vec::from_iter(self.connected_module.keys().cloned());
    }
}

fn get_module_validator(module_type: char, ) -> Box<dyn interface::ModuleValueValidator> {
    // TODO switch back to a match but i was having issue with match :(
    if module_type == 'A' {
        return Box::new(aaa::AAAValidator::new());
    } else if module_type == 'S' {
        return Box::new(aas::AASValidator::new());
    } else if module_type == 'P' {
        return Box::new(aap::AAPValidator::new());
    } else if module_type == 'B' {
        return Box::new(AABValidator::new());
    } else {
        panic!("its a panic no validator found for type {}", module_type);
    }
}


fn extract_module_id(topic_name: &String) -> String {
    let pieces: Vec<&str> = topic_name.split("/").collect();
    let last = pieces.get(pieces.len() -1).unwrap();
    return String::from(last.clone());
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
    if let Err(err) = sender_socket.send((String::from(format!("/m/{}/state", id)), Box::new(send_state))) {
        log::error!("error sending message for module state {:?}", err);
    }
}

#[inline]
fn handle_module_state(
    manager: & mut MainboardModuleStateManager,
    state: & mut ModuleStateChangeEvent,
    sender_comboard_config: & Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: &store::ModuleStateStore,
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
) -> () {
    if state.state == true {
            let type_character_option = state.id.chars().nth(2);
            if type_character_option.is_none() {
                log::error!("module without id just connected on port {}", state.port);
                return;
            }
            log::debug!("module connected {} at {}", state.id.as_str(), state.port);
            let t = state.id.chars().nth(2).unwrap();
            let validator = get_module_validator(t);
            manager.connected_module.insert(state.id.clone(), MainboardConnectedModule{
                port: state.port,
                id: state.id.clone(),
                handler_map: std::collections::HashMap::new(),
                last_value: None,
                validator: validator,
            });

            send_module_state(state.id.as_str(), state.port, true, sender_socket);


            let config = store.get_module_config(&state.id);
            if config.is_some() {

                // TODO implement fonction to handle not byte but structure directly
                let module_mut_ref = manager.connected_module.get_mut(state.id.as_str()).unwrap();
                let bytes = Arc::new(config.unwrap().write_to_bytes().unwrap());
                match module_mut_ref.validator.apply_parse_config(state.port, t, bytes, sender_comboard_config, &mut module_mut_ref.handler_map) {
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

    let reference_connected_module_option = manager.get_module_at_index_mut(value.port);
    if reference_connected_module_option.is_none() {
        log::error!("receive value for port {} but module is not in the store", value.port);
        return;
    }
    let reference_connected_module = reference_connected_module_option.unwrap();

    let on_change = |value| {
       sender_socket
                .send((String::from(format!("/m/{}/data", reference_connected_module.id)), value))
                .expect("Failed to send !!!");

    };

    match reference_connected_module.validator.convert_to_value(value) {

        Ok(sensor_value) => {
            if reference_connected_module.last_value.is_some() {
                let change = reference_connected_module.validator.have_data_change(&sensor_value, reference_connected_module.last_value.as_ref().unwrap());
                if change.0 == true {
                    on_change(sensor_value);
                    if let Ok(previous_value) = reference_connected_module.validator.convert_to_value(value) {
                        reference_connected_module.last_value = Some(previous_value);

                        log::debug!("data have changed for {}", reference_connected_module.id.as_str());

                        if change.1.len() > 0 {
                            let module_value_change = alarm::model::ModuleValueChange::<i32>{
                                module_id: reference_connected_module.id.clone(),
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
                if let Ok(previous_value) = reference_connected_module.validator.convert_to_value(value) {
                    reference_connected_module.last_value = Some(previous_value);
                }
            }
        },
        Err(e) => log::error!("{}", e),
    }
}



fn handle_module_config(
    topic: &String,
    data: Arc<Vec<u8>>,
    manager: & mut MainboardModuleStateManager,
    sender_config: & Sender<crate::comboard::imple::interface::Module_Config>,
    _sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: &store::ModuleStateStore,
) -> Result<(), interface::ModuleError> {
    let id = crate::utils::mqtt::last_element_path(topic);
    let module_ref_option = manager.connected_module.get_mut(id.as_str());

    if let Some(module_ref) = module_ref_option {

        let t = module_ref.id.chars().nth(2).unwrap();

        match module_ref.validator.apply_parse_config(module_ref.port, t, data, &sender_config, &mut module_ref.handler_map) {
            Ok((config, config_comboard)) => {
                store.store_module_config(&id, config);
                sender_config.send(config_comboard).unwrap();
            },
            Err(e) => log::error!("{}", e)
        }
        tokio::task::spawn(async {});
    } else {
        return Err(interface::ModuleError::not_found(&id));
    }

    return Ok(());
}

fn handle_remove_module_config(
    topic: &String,
    data: Arc<Vec<u8>>,
    manager: & mut MainboardModuleStateManager,
    sender_config: & Sender<crate::comboard::imple::interface::Module_Config>,
    _sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: &store::ModuleStateStore,
) -> Result<(), interface::ModuleError> {
    let id = crate::utils::mqtt::last_element_path(topic);
    let module_ref_option = manager.connected_module.get_mut(id.as_str());
    if let Some(module_ref) = module_ref_option {
        module_ref.validator.remove_config().unwrap();
        store.delete_module_config(&id).unwrap();
    } else {
        return Err(interface::ModuleError::not_found(&id));
    }

    return Ok(());
}


fn handle_sync_request(
    manager: & mut MainboardModuleStateManager,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
) -> Result<(), interface::ModuleError>  {
    log::debug!("send sync request to the cloud");
    for (k,v) in manager.connected_module.iter() {
        send_module_state(k, v.port, true, sender_socket);
    }

    return Ok(());
}

fn handle_add_alarm(
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
    data: Arc<Vec<u8>>,
) -> Result<(), interface::ModuleError>  {
    let field_alarm = FieldAlarm::parse_from_bytes(&data).unwrap();
    alarm_store.add_alarm_field(&field_alarm).unwrap();
    alarm_validator.register_field_alarm(field_alarm).unwrap();

    return Ok(());
}

fn handle_remove_alarm(
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
    data: Arc<Vec<u8>>,
) -> Result<(), interface::ModuleError> {
    let field_alarm = FieldAlarm::parse_from_bytes(&data).unwrap();
    alarm_store.remove_alarm_field(&field_alarm).unwrap();
    alarm_validator.deregister_field_alarm(field_alarm).unwrap();

    return Ok(());
}

fn handle_validator_command(
    cmd: &str,
    module_id: &String,
    sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
    manager: & mut MainboardModuleStateManager,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    data: std::sync::Arc<Vec<u8>>,
) -> Result<std::option::Option<Vec<ModuleStateCmd>>, interface::ModuleError> {
    if let Some(module) = manager.connected_module.get_mut(module_id) {
        return module.validator.handle_command_validator(cmd, module_id,data, sender_response, sender_socket);
    } else {
        return Err(interface::ModuleError::not_found(module_id));
    }
}

fn handle_module_command(
    cmd: &str,
    topic: &String,
    data: std::sync::Arc<Vec<u8>>,
    sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
    manager: & mut MainboardModuleStateManager,
    store: & store::ModuleStateStore,
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
    sender_config: & Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    virtual_relay_store: &mut relay::virtual_relay::store::VirtualRelayStore,
) -> () {
    let result = match cmd {
        "sync" => handle_sync_request(manager, &sender_socket),
        "mconfig" => handle_module_config(topic, data, manager, &sender_config, &sender_socket, &store),
        "rmconfig" => handle_remove_module_config(topic, data, manager, &sender_config, &sender_socket, &store),
        "aAl" => handle_add_alarm(alarm_validator, &alarm_store, data),
        "rAl" => handle_remove_alarm(alarm_validator, &alarm_store, data),
        "addVr" => relay::virtual_relay::handler::handle_virtual_relay(
            data,  &sender_config, &sender_socket, store, virtual_relay_store, manager,
        ),
        "configVr" => relay::virtual_relay::handler::handle_apply_config_virtual_relay(
            topic, data, sender_config, sender_socket, store, virtual_relay_store, manager,
        ),
        "rmVr" => relay::virtual_relay::handler::handle_delete_virtual_relay(
             topic, data, sender_config, sender_socket, store, virtual_relay_store, manager,
        ),
        _ => {
            let module_id = extract_module_id(topic);
            match handle_validator_command(cmd,&module_id, sender_response,manager, &sender_socket, data) {
                Ok(option_cmd) => {
                    if let Some(cmds) = option_cmd {
                        cmds.into_iter().for_each(|cmd| {
                            handle_module_command(
                                cmd.cmd,
                                &cmd.topic,
                                cmd.data,
                                sender_response,
                                manager,
                                store,
                                alarm_validator,
                                alarm_store,
                                sender_config,
                                sender_socket,
                                virtual_relay_store,
                            );
                        });
                        Ok(())
                    } else {
                        // end of chain return
                        Ok(())
                    }
                },
                Err(_e) => {
                    log::debug!("failed to execute module validator command {} for {}", cmd ,module_id);
                    Err(interface::ModuleError::not_found("cmd"))
                }
            }
        },
    };

    let mut action_respose = crate::protos::message::ActionResponse::new();
    match result {
        Ok(()) => {
            action_respose.status = 0;
        },
        Err(_module_error) => {
            action_respose.status = 500;
        }
    }
    
    sender_response.send(action_respose);
}


pub fn module_state_task(
    sender_socket: Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: store::ModuleStateStore,
    alarm_store: alarm::store::ModuleAlarmStore,
) -> tokio::task::JoinHandle<()> {

    let sender_config = CHANNEL_CONFIG.0.lock().unwrap().clone();
        

    return tokio::spawn(async move {
        let mut manager = MainboardModuleStateManager{
            connected_module: HashMap::new(),
        };
    
        let mut alarm_validator = alarm::validator::AlarmFieldValidator::new();

        let mut virtual_relay_store = relay::virtual_relay::store::VirtualRelayStore::new(
            alarm_store.conn.clone()
        );

        let receiver_state = CHANNEL_STATE.1.lock().unwrap();
        let receiver_value = CHANNEL_VALUE.1.lock().unwrap();


        loop {
            {
                let receive = receiver_state.try_recv();
                if receive.is_ok() {
                    let mut state = receive.unwrap();
                    handle_module_state(& mut manager, & mut state, &sender_config, &sender_socket, &store, & mut alarm_validator,&alarm_store);
                    on_module_state_changed_virtual_relays(state.state, &sender_config, &sender_socket, &store, &mut virtual_relay_store, &mut manager).unwrap();
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
                    handle_module_command(
                        cmd.cmd,
                        &cmd.topic,
                        cmd.data,
                        &cmd.sender,
                        &mut manager,
                        &store,
                        &mut alarm_validator,
                        &alarm_store,
                        &sender_config,
                        &sender_socket,
                        &mut virtual_relay_store,
                    );
                }
            }
        }
    });
}
