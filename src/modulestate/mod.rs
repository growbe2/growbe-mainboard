
pub mod aaa;
pub mod aas;
pub mod aap;
pub mod aab;
pub mod ppo;
pub mod pac;
pub mod pal;
pub mod ppr;
pub mod pcs;
pub mod ccs;
pub mod css;

pub mod store;
pub mod relay;
pub mod interface;
pub mod alarm;
pub mod actor;


use crate::{comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent}};
use crate::comboard::imple::channel::*;
use crate::protos::alarm::FieldAlarm;
use interface::ModuleStateCmd;
use lazy_static::lazy_static;
use regex::Regex;
use protobuf::Message;
use std::{collections::HashMap};
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{Receiver, Sender,};
use aab::AABValidator;

use self::{relay::virtual_relay::handler::on_module_state_changed_virtual_relays, pac::PACValidator, ppo::PPOValidator, ppr::PPRValidator, pal::PALValidator, pcs::PCSValidator, ccs::CCSValidator, css::CSSValidator};

lazy_static! {
    pub static ref CHANNEL_MODULE_STATE_CMD: (Mutex<Sender<ModuleStateCmd>>, Mutex<Receiver<ModuleStateCmd>>) = {
        let (sender, receiver) = std::sync::mpsc::channel::<ModuleStateCmd>();
        return (Mutex::new(sender), Mutex::new(receiver));
    };

    static ref REGEX_MODULE_ID: Regex = Regex::new("[A-Z]{3}[A-Z0-9]{9}").unwrap();
}


pub struct MainboardConnectedModule {
    pub port: i32,
    pub id: String,
    pub board: String,
    pub board_addr: String,
    pub handler_map: std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
    pub last_value: Option<Box<dyn interface::ModuleValueParsable>>,
    pub validator: Box<dyn interface::ModuleValueValidator>,
}

pub struct MainboardModuleStateManager {
    pub connected_module: HashMap<String, MainboardConnectedModule>,
}


impl MainboardModuleStateManager {
    fn get_module_at_index(&self, board: &String, board_addr: &String, port: i32) -> Option<&MainboardConnectedModule> {
        for (_, v) in self.connected_module.iter() {
            if v.port == port && v.board == *board && v.board_addr == *board_addr {
                return Some(&v);
            }
        }
        return None;
    }
    // cheap hack plz can i do better
    fn get_module_at_index_mut(&mut self, board: &String, board_addr: &String, port: i32) -> Option<&mut MainboardConnectedModule> {
        let mut id: String = String::from("");
        {
            if let Some(its_a_me_variable) = self.get_module_at_index(board, board_addr, port) {
                id = its_a_me_variable.id.clone();
            }
        }
        return self.connected_module.get_mut(&id);
    }

    fn get_connected_modules(&self) -> Vec<String> {
        return Vec::from_iter(self.connected_module.keys().cloned());
    }
}

fn get_module_validator(module_type: &str ) -> Result<Box<dyn interface::ModuleValueValidator>, interface::ModuleError> {
    if module_type == "AAA" {
        return Ok(Box::new(aaa::AAAValidator::new()));
    } else if module_type == "AAS" {
        return Ok(Box::new(aas::AASValidator::new()));
    } else if module_type == "AAP" {
        return Ok(Box::new(aap::AAPValidator::new()));
    } else if module_type == "AAB" {
        return Ok(Box::new(AABValidator::new()));
    } else if module_type == "PAC" {
        return Ok(Box::new(PACValidator::new()));
    } else if module_type == "PPO" {
        return Ok(Box::new(PPOValidator::new()));
    } else if module_type == "PPR" {
        return Ok(Box::new(PPRValidator::new()));
    } else if module_type == "PAL" {
        return Ok(Box::new(PALValidator::new()));
    } else if module_type == "PCS" {
        return Ok(Box::new(PCSValidator::new()));
    } else if module_type == "CCS" {
        return Ok(Box::new(CCSValidator::new()));
    } else if module_type == "CSS" {
        return Ok(Box::new(CSSValidator::new()));
    } else {
        return Err(interface::ModuleError::new().message("cannot find validator for module type".to_string()));
    }
}


fn extract_module_id(topic_name: &String) -> String {
    let pieces: Vec<&str> = topic_name.split("/").collect();
    let last = pieces.get(pieces.len() -1).unwrap();
    return String::from(last.clone());
}

fn valid_module_id(module_id: &String) -> bool {
    return REGEX_MODULE_ID.is_match(module_id);
}

fn send_module_state(
    id: &str,
    port: i32,
    state: bool,
    board: &String,
    board_addr: &String,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
) -> () {
    let mut send_state = crate::protos::module::ModuleData::new();
    send_state.id = String::from(id);
    send_state.plug = state;
    send_state.atIndex = port;
    send_state.board = board.clone();
    send_state.boardAddr = board_addr.clone();
    if let Err(err) = sender_socket.send((String::from(format!("/m/{}/state", id)), Box::new(send_state))) {
        log::error!("error sending message for module state {:?}", err);
    }
}

#[inline]
fn handle_module_state(
    manager: & mut MainboardModuleStateManager,
    state: & mut ModuleStateChangeEvent,
    sender_comboard_config: &ComboardSenderMapReference,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: &store::ModuleStateStore,
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
) -> () {
    if !valid_module_id(&state.id) {
        if state.state == true {
            log::error!("receive state changed from invalid ID {} state {}", state.id, state.state);
        }
        return;
    }
    if state.state == true {
            let type_character_option = state.id.chars().nth(2);
            if type_character_option.is_none() {
                log::error!("module without id just connected on port {}", state.port);
                return;
            }
            log::debug!("module connected {} at {}", state.id.as_str(), state.port);
            let t = &state.id[..3];
            let validator_result = get_module_validator(t);
            if validator_result.is_err() {
                //log::error!("{}", validator_result.unwrap_err().message);
                log::error!("cannot find a validator for module {}", state.id);
                return;
            }
            let validator = validator_result.unwrap();
            manager.connected_module.insert(state.id.clone(), MainboardConnectedModule{
                port: state.port,
                id: state.id.clone(),
                board: state.board.clone(),
                board_addr: state.board_addr.clone(),
                handler_map: std::collections::HashMap::new(),
                last_value: None,
                validator: validator,
            });

            send_module_state(state.id.as_str(), state.port, true, &state.board, &state.board_addr, sender_socket);


            let config = store.get_module_config(&state.id);
            if config.is_some() {
                // TODO implement fonction to handle not byte but structure directly
                let module_mut_ref = manager.connected_module.get_mut(state.id.as_str()).unwrap();
                let bytes = Arc::new(config.unwrap().write_to_bytes().unwrap());

                let sender_config = sender_comboard_config.get_sender(ComboardAddr { imple: module_mut_ref.board.clone(), addr: module_mut_ref.board_addr.clone() }).unwrap();
                match module_mut_ref.validator.apply_parse_config(state.port, t, bytes, &sender_config, &mut module_mut_ref.handler_map) {
                    Ok((_config, config_comboard)) => sender_config.send(config_comboard).unwrap(),
                    Err(e) => log::error!("validation error {}", e),
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
            send_module_state(state.id.as_str(), state.port, false, &state.board, &state.board_addr, sender_socket);
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
    let reference_connected_module_option = manager.get_module_at_index_mut(&value.board, &value.board_addr, value.port);
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
        Err(e) => log::error!("convert to value error : {}", e),
    }
}



fn handle_module_config(
    topic: &String,
    data: Arc<Vec<u8>>,
    manager: & mut MainboardModuleStateManager,
    sender_config: &ComboardSenderMapReference,
    _sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: &store::ModuleStateStore,
) -> Result<(), interface::ModuleError> {
    let id = crate::utils::mqtt::last_element_path(topic);
    let module_ref_option = manager.connected_module.get_mut(id.as_str());

    if let Some(module_ref) = module_ref_option {

        let t = &module_ref.id[..3];

        if let Ok(sender_config) = sender_config.get_sender(ComboardAddr { imple: module_ref.board.clone(), addr: module_ref.board_addr.clone() }) {
            match module_ref.validator.apply_parse_config(module_ref.port, t, data, &sender_config, &mut module_ref.handler_map) {
                Ok((config, config_comboard)) => {
                    sender_config.send(config_comboard).unwrap();
                    store.store_module_config(&id, config);
                },
                Err(e) => log::error!("{}", e)
            }
            tokio::task::spawn(async {});
        } else {
            return Err(interface::ModuleError::sender_not_found(&id))
        }
    } else {
        return Err(interface::ModuleError::not_found(&id));
    }

    return Ok(());
}

fn handle_remove_module_config(
    topic: &String,
    _data: Arc<Vec<u8>>,
    manager: & mut MainboardModuleStateManager,
    _sender_config: &ComboardSenderMapReference,
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
        send_module_state(v.id.as_str(), v.port, true, &v.board, &v.board_addr, sender_socket);
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


fn handle_update_alarm(
    alarm_validator: & mut alarm::validator::AlarmFieldValidator,
    alarm_store: & alarm::store::ModuleAlarmStore,
    data: Arc<Vec<u8>>,
) -> Result<(), interface::ModuleError>  {
    let field_alarm = FieldAlarm::parse_from_bytes(&data).unwrap();
    alarm_store.update_alarm_field(&field_alarm).unwrap();
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
    sender_config: &ComboardSenderMapReference,
    sender_socket: & Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    virtual_relay_store: &mut relay::virtual_relay::store::VirtualRelayStore,
) -> () {
    let result = match cmd {
        "sync" => handle_sync_request(manager, &sender_socket),
        "mconfig" => handle_module_config(topic, data, manager, &sender_config, &sender_socket, &store),
        "rmconfig" => handle_remove_module_config(topic, data, manager, &sender_config, &sender_socket, &store),
        "aAl" => handle_add_alarm(alarm_validator, &alarm_store, data),
        "rAl" => handle_remove_alarm(alarm_validator, &alarm_store, data),
        "uAl" => handle_update_alarm(alarm_validator, &alarm_store, data),
        "addVr" => relay::virtual_relay::handler::handle_virtual_relay(
            data,  &sender_config, &sender_socket, store, virtual_relay_store, manager,
        ),
        "vrconfig" => relay::virtual_relay::handler::handle_apply_config_virtual_relay(
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
                Err(e) => {
                    Err(e)
                }
            }
        },
    };

    let mut action_respose = crate::protos::message::ActionResponse::new();
    match result {
        Ok(()) => {
            action_respose.status = 0;
        },
        Err(module_error) => {
            action_respose.status = module_error.status;
            action_respose.msg = module_error.message;
        }
    }
    
    sender_response.send(action_respose).unwrap();
}


pub fn module_state_task(
    sender_socket: Sender<(String, Box<dyn interface::ModuleValueParsable>)>,
    store: store::ModuleStateStore,
    sender_config: ComboardSenderMapReference,
    alarm_store: alarm::store::ModuleAlarmStore,
) -> tokio::task::JoinHandle<()> {

        

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
