use std::sync::mpsc::{Receiver, Sender, self};
use std::sync::{Arc, Mutex};

use crate::comboard::imple::channel::ComboardAddr;
use crate::comboard::imple::channel::ComboardSenderMapReference;
use crate::mainboardstate::error::MainboardError;
use crate::protos::alarm::FieldAlarm;
use crate::protos::message::ActionResponse;
use crate::utils::mqtt::extract_module_id;

use protobuf::Message;

use super::interface::{ModuleError, ModuleStateCmd};
use super::{handle_state::send_module_state, state_manager::MainboardModuleStateManager};

lazy_static::lazy_static! {
    pub static ref CHANNEL_MODULE_STATE_CMD: (
        Mutex<Sender<ModuleStateCmd>>,
        Mutex<Receiver<ModuleStateCmd>>
    ) = {
        let (sender, receiver) = std::sync::mpsc::channel::<ModuleStateCmd>();
        return (Mutex::new(sender), Mutex::new(receiver));
    };
}
fn handle_module_config(
    topic: &String,
    data: Arc<Vec<u8>>,
    manager: &mut MainboardModuleStateManager,
    sender_config: &ComboardSenderMapReference,
    _sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    store: &super::store::ModuleStateStore,
) -> Result<(), MainboardError> {
    let id = crate::utils::mqtt::last_element_path(topic).ok_or(
        MainboardError::new().message("failed to get last element from mqtt topic".to_string()),
    )?;

    let module_ref_option = manager.connected_module.get_mut(id.as_str());

    if let Some(module_ref) = module_ref_option {
        let t = &module_ref.id[..3];

        if let Ok(sender_config) = sender_config.get_sender(ComboardAddr {
            imple: module_ref.board.clone(),
            addr: module_ref.board_addr.clone(),
        }) {
            match module_ref.validator.apply_parse_config(
                module_ref.port,
                t,
                data,
                &sender_config,
                &mut module_ref.handler_map,
            ) {
                Ok((config, config_comboard)) => {
                    store.store_module_config(&id, config)?;
                    sender_config
                        .send(config_comboard)
                        .map_err(|x| MainboardError::from_error(x.to_string()))?;
                }
                Err(e) => return Err(e.into()),
            }
            tokio::task::spawn(async {});
        } else {
            return Err(super::interface::ModuleError::sender_not_found(&id).into());
        }
    } else {
        return Err(super::interface::ModuleError::not_found(&id).into());
    }

    return Ok(());
}

fn handle_remove_module_config(
    topic: &String,
    _data: Arc<Vec<u8>>,
    manager: &mut MainboardModuleStateManager,
    _sender_config: &ComboardSenderMapReference,
    _sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    store: &super::store::ModuleStateStore,
) -> Result<(), MainboardError> {
    let id = crate::utils::mqtt::last_element_path(topic).ok_or(
        ModuleError::new().message("failed to get last element from mqtt topic".to_string()),
    )?;
    let module_ref_option = manager.connected_module.get_mut(id.as_str());
    if let Some(module_ref) = module_ref_option {
        module_ref.validator.remove_config()?;
        store.delete_module_config(&id)?;
    } else {
        return Err(super::interface::ModuleError::not_found(&id).into());
    }

    return Ok(());
}

fn handle_sync_request(
    manager: &mut MainboardModuleStateManager,
    sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
) -> Result<(), MainboardError> {
    log::debug!("send sync request to the cloud");
    for (_k, v) in manager.connected_module.iter() {
        send_module_state(
            v.id.as_str(),
            v.port,
            true,
            &v.board,
            &v.board_addr,
            sender_socket,
        )?;
    }

    return Ok(());
}

fn handle_add_alarm(
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    data: Arc<Vec<u8>>,
) -> Result<(), MainboardError> {
    let field_alarm = FieldAlarm::parse_from_bytes(&data)?;
    alarm_store.add_alarm_field(&field_alarm)?;
    alarm_validator.register_field_alarm(field_alarm, None)?;

    return Ok(());
}

fn handle_update_alarm(
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    data: Arc<Vec<u8>>,
) -> Result<(), MainboardError> {
    let field_alarm = FieldAlarm::parse_from_bytes(&data)?;
    alarm_store.update_alarm_field(&field_alarm)?;
    alarm_validator.register_field_alarm(field_alarm, None)?;

    return Ok(());
}

fn handle_remove_alarm(
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    data: Arc<Vec<u8>>,
) -> Result<(), MainboardError> {
    let field_alarm = FieldAlarm::parse_from_bytes(&data)?;
    alarm_store.remove_alarm_field(&field_alarm)?;
    alarm_validator.deregister_field_alarm(field_alarm)?;

    return Ok(());
}

fn handle_validator_command(
    cmd: &str,
    module_id: &String,
    sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
    manager: &mut MainboardModuleStateManager,
    sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    data: std::sync::Arc<Vec<u8>>,
) -> Result<std::option::Option<Vec<ModuleStateCmd>>, super::interface::ModuleError> {
    if let Some(module) = manager.connected_module.get_mut(module_id) {
        return module.validator.handle_command_validator(
            cmd,
            module_id,
            data,
            sender_response,
            sender_socket,
        );
    } else {
        return Err(super::interface::ModuleError::not_found(module_id));
    }
}

pub fn handle_module_command(
    cmd: &str,
    topic: &String,
    data: std::sync::Arc<Vec<u8>>,
    sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
    manager: &mut MainboardModuleStateManager,
    store: &super::store::ModuleStateStore,
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    sender_config: &ComboardSenderMapReference,
    sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    virtual_relay_store: &mut super::relay::virtual_relay::store::VirtualRelayStore,
) -> Result<(), MainboardError> {
    let result: Result<(), MainboardError> = match cmd {
        "sync" => handle_sync_request(manager, &sender_socket),
        "mconfig" => {
            handle_module_config(topic, data, manager, &sender_config, &sender_socket, &store)
        }
        "rmconfig" => handle_remove_module_config(
            topic,
            data,
            manager,
            &sender_config,
            &sender_socket,
            &store,
        ),
        "aAl" => handle_add_alarm(alarm_validator, &alarm_store, data),
        "rAl" => handle_remove_alarm(alarm_validator, &alarm_store, data),
        "uAl" => handle_update_alarm(alarm_validator, &alarm_store, data),
        "addVr" => super::relay::virtual_relay::handler::handle_virtual_relay(
            data,
            &sender_config,
            &sender_socket,
            store,
            virtual_relay_store,
            manager,
        ),
        "vrconfig" => super::relay::virtual_relay::handler::handle_apply_config_virtual_relay(
            topic,
            data,
            sender_config,
            sender_socket,
            store,
            virtual_relay_store,
            manager,
        ),
        "rmVr" => super::relay::virtual_relay::handler::handle_delete_virtual_relay(
            topic,
            data,
            sender_config,
            sender_socket,
            store,
            virtual_relay_store,
            manager,
        ),
        _ => {
            let module_id = extract_module_id(topic);
            match handle_validator_command(
                cmd,
                &module_id,
                sender_response,
                manager,
                &sender_socket,
                data,
            ) {
                Ok(option_cmd) => {
                    if let Some(cmds) = option_cmd {
                        cmds.into_iter().for_each(|cmd| {
                            if let Err(err) = handle_module_command(
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
                            ) {
                                log::error!("failed to handle_module_command : {:?}", err);
                            }
                        });
                        Ok(())
                    } else {
                        // end of chain return
                        Ok(())
                    }
                }
                Err(e) => Err(e.into()),
            }
        }
    };

    let mut action_respose = crate::protos::message::ActionResponse::new();
    match result {
        Ok(()) => {
            action_respose.status = 0;
            if let Err(err) = sender_response.send(action_respose) {
                log::error!("failed to send action response : {:?}", err);
            }
            return Ok(());
        }
        Err(mainboard_error) => {
            action_respose.status = 500;
            action_respose.msg = mainboard_error.message.clone();
            if let Err(err) = sender_response.send(action_respose) {
                log::error!("failed to send action response : {:?}", err);
            }
            return Err(mainboard_error);
        }
    }
}
