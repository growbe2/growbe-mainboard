use std::sync::mpsc::Sender;
use std::sync::Arc;
use regex::Regex;

use crate::comboard::imple::channel::ComboardAddr;
use crate::mainboardstate::error::MainboardError;
use crate::comboard::imple::{
    interface::ModuleStateChangeEvent,
    channel::ComboardSenderMapReference
};
use super::state_manager::{MainboardConnectedModule, MainboardModuleStateManager};

lazy_static::lazy_static! {
    static ref REGEX_MODULE_ID: Regex = Regex::new("[A-Z]{3}[A-Z0-9]{9}").unwrap();
}

pub fn send_module_state(
    id: &str,
    port: i32,
    state: bool,
    board: &String,
    board_addr: &String,
    sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
) -> Result<(), MainboardError> {
    let mut send_state = crate::protos::module::ModuleData::new();
    send_state.id = String::from(id);
    send_state.plug = state;
    send_state.atIndex = port;
    send_state.board = board.clone();
    send_state.boardAddr = board_addr.clone();
    sender_socket.send((
        String::from(format!("/m/{}/state", id)),
        Box::new(send_state),
    ))?;
    Ok(())
}


fn get_module_validator(
    module_type: &str,
) -> Result<Box<dyn super::interface::ModuleValueValidator>, MainboardError> {
    if module_type == "AAA" {
        return Ok(Box::new(super::aaa::AAAValidator::new()));
    } else if module_type == "AAS" {
        return Ok(Box::new(super::aas::AASValidator::new()));
    } else if module_type == "AAP" {
        return Ok(Box::new(super::aap::AAPValidator::new()));
    } else if module_type == "AAB" {
        return Ok(Box::new(super::aab::AABValidator::new()));
    } else if module_type == "PAC" {
        return Ok(Box::new(super::pac::PACValidator::new()));
    } else if module_type == "PPO" {
        return Ok(Box::new(super::ppo::PPOValidator::new()));
    } else if module_type == "PPR" {
        return Ok(Box::new(super::ppr::PPRValidator::new()));
    } else if module_type == "PAL" {
        return Ok(Box::new(super::pal::PALValidator::new()));
    } else if module_type == "PCS" {
        return Ok(Box::new(super::pcs::PCSValidator::new()));
    } else if module_type == "CCS" {
        return Ok(Box::new(super::ccs::CCSValidator::new()));
    } else if module_type == "CSS" {
        return Ok(Box::new(super::css::CSSValidator::new()));
    } else {
        return Err(MainboardError::new()
            .message("cannot find validator for module type".to_string()));
    }
}

fn valid_module_id(module_id: &String) -> bool {
    return REGEX_MODULE_ID.is_match(module_id);
}


#[inline]
pub fn handle_module_state(
    manager: &mut MainboardModuleStateManager,
    state: &mut ModuleStateChangeEvent,
    sender_comboard_config: &ComboardSenderMapReference,
    sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    store: &super::store::ModuleStateStore,
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
) -> Result<(), MainboardError> {
    if !valid_module_id(&state.id) {
        if state.state == true {
            log::error!(
                "receive state changed from invalid ID {} state {}",
                state.id,
                state.state
            );
        }
        return Err(MainboardError::from_error(format!("id receive is invalid {}", state.id)));
    }
    if state.state == true {
        // VALIDATE ID AND REGISER CONNECTER Module
        //
        let type_character_option = state.id.chars().nth(2);
        if type_character_option.is_none() {
            return Err(MainboardError::from_error(format!("module without id just connect on port {}", state.port)));
        }
        log::debug!("module connected {} at {}", state.id.as_str(), state.port);
        let t = &state.id[..3];
        let validator = get_module_validator(t)?;
        manager.connected_module.insert(
            state.id.clone(),
            MainboardConnectedModule {
                port: state.port,
                id: state.id.clone(),
                board: state.board.clone(),
                board_addr: state.board_addr.clone(),
                handler_map: std::collections::HashMap::new(),
                last_value: None,
                validator,
            },
        );

        // SEND THE STATE TO THE CLOUD
        if let Err(err) = send_module_state(
            state.id.as_str(),
            state.port,
            true,
            &state.board,
            &state.board_addr,
            sender_socket,
        ) {
            log::error!("failed to send module state : {:?}", err);
        }


        // BLOCK TO APPLY CONFIG
        //
        let config = store.get_module_config(&state.id);
        if config.is_some() {
            // TODO implement fonction to handle not byte but structure directly
            if let Some(module_mut_ref) = manager.connected_module.get_mut(state.id.as_str()) {
                let bytes = Arc::new(config.unwrap().write_to_bytes().unwrap());

                let sender_config = sender_comboard_config
                    .get_sender(ComboardAddr {
                        imple: module_mut_ref.board.clone(),
                        addr: module_mut_ref.board_addr.clone(),
                    })?;
                match module_mut_ref.validator.apply_parse_config(
                    state.port,
                    t,
                    bytes,
                    &sender_config,
                    &mut module_mut_ref.handler_map,
                ) {
                    Ok((_config, config_comboard)) => sender_config.send(config_comboard).unwrap(),
                    // TODO: Send message to cloud saying we failed to apply config
                    Err(e) => log::error!("validation error to apply the config {}", e),
                }
                tokio::task::spawn(async {});
            } else {
                // TODO: Send message to cloud saying we failed to apply config
                log::error!("failed to get module_ref to apply config");
            }
        }

        // BLOCK TO HANDLE ALARMS
        //
        match alarm_store.get_alarm_for_module(&state.id.clone()) {
            Ok(mut alarms) => {
                log::info!("loading {} alarms for {}", alarms.len(), state.id.as_str());
                for _n in 0..alarms.len() {
                    if let Some((alarm, state)) = alarms.pop() {
                        if let Err(err) = alarm_validator.register_field_alarm(alarm, state) {
                            log::error!("failed to register alarm : {:?}", err);
                        }
                    } else {
                        log::error!("failed to get next alarm in list");
                    }
                }
            },
            Err(err) => log::error!("failed to get alarms for modules : {:?}", err),
        }
    }
    if state.state == false {
        log::debug!(
            "Module disconnected {} at {}",
            state.id.as_str(),
            state.port
        );
        // remove from module map
        let connected_module = manager.connected_module.remove(state.id.as_str());
        // clear task
        if let Some(connected_module) = connected_module {
            connected_module
                .handler_map
                .iter()
                .for_each(|module| module.1.cancel());

            if let Err(err) = send_module_state(
                state.id.as_str(),
                state.port,
                false,
                &state.board,
                &state.board_addr,
                sender_socket,
            ) {
                log::error!("failed to send module state {:?}", err);
            }

            let alarms_result = alarm_store.get_alarm_for_module(&state.id.clone());
            if let Ok(mut alarms) = alarms_result {
                log::info!("removing {} alarms for {}", alarms.len(), state.id.as_str());
                for _n in 0..alarms.len() {
                    if let Some((alarm, _)) = alarms.pop() {
                        if let Err(err) = alarm_validator.deregister_field_alarm(alarm) {
                            log::error!("failed to dereigster field alarm : {:?}", err);
                        }
                    }
                }
            }
        } else {
            log::error!("failed to get disconnecting module from module manager");
        }
    }

    return Ok(());
}


