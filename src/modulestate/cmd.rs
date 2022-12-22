use tokio::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::comboard::imple::channel::ComboardAddr;
use crate::comboard::imple::channel::ComboardSenderMapReference;
use crate::mainboardstate::error::MainboardError;
use crate::protos::alarm::FieldAlarm;
use crate::protos::env_controller::EnvironmentControllerConfiguration;
use crate::protos::module::Actor;
use crate::utils::mqtt::extract_module_id;

use protobuf::Message;

use super::controller::store::EnvControllerStore;
use super::interface::ModuleMsg;
use super::interface::{ModuleError, ModuleStateCmd};
use super::{handle_state::send_module_state, state_manager::MainboardModuleStateManager};

fn apply_module_config(
    id: &str,
    data: Arc<Vec<u8>>,
    manager: &mut MainboardModuleStateManager,
    sender_config: &ComboardSenderMapReference,
    store: &super::store::ModuleStateStore,
    suffix: &str,
    from_actor: &Actor,
) -> Result<(), MainboardError> {
    let module_ref_option = manager.connected_module.get_mut(id);

    if let Some(module_ref) = module_ref_option {
        let t = &module_ref.id[..3];

        if let Ok(sender_config) = sender_config.get_sender(ComboardAddr {
            imple: module_ref.board.clone(),
            addr: module_ref.board_addr.clone(),
        }) {

            let t = if suffix.is_empty() { t.to_string() } else { format!("{}:{}", t, suffix) };

            match module_ref.validator.apply_parse_config(
                module_ref.port,
                &t,
                data,
                &sender_config,
                &mut module_ref.handler_map,
                from_actor.clone(),
            ) {
                Ok((config, config_comboard)) => {
                    store.store_module_config(&(id.into()), config)?;
                    sender_config
                        .try_send(config_comboard)
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


fn handle_module_config(
    topic: &String,
    data: Arc<Vec<u8>>,
    manager: &mut MainboardModuleStateManager,
    sender_config: &ComboardSenderMapReference,
    _sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    store: &super::store::ModuleStateStore,
    from_actor: &Actor,
) -> Result<(), MainboardError> {
    let id = crate::utils::mqtt::last_element_path(topic).ok_or(
        MainboardError::new().message("failed to get last element from mqtt topic".to_string()),
    )?;

    return apply_module_config(&id, data, manager, sender_config, store, "", from_actor);
}

fn handle_pmodule_config(
    topic: &String,
    data: Arc<Vec<u8>>,
    manager: &mut MainboardModuleStateManager,
    sender_config: &ComboardSenderMapReference,
    _sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    store: &super::store::ModuleStateStore,
    from_actor: &Actor,
) -> Result<(), MainboardError> {
    let (id, property) = crate::utils::mqtt::last_2_element_path(topic).ok_or(
        MainboardError::new().message("failed to get last element from mqtt topic".to_string()),
    )?;

    return apply_module_config(&id, data, manager, sender_config, store, &property, from_actor);
}

fn handle_remove_module_config(
    topic: &String,
    _data: Arc<Vec<u8>>,
    manager: &mut MainboardModuleStateManager,
    _sender_config: &ComboardSenderMapReference,
    _sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    store: &super::store::ModuleStateStore,
    from_actor: &Actor,
) -> Result<(), MainboardError> {
    // TODO: for actor need to validate that i can delete a config if all property are owned by me
    let id = crate::utils::mqtt::last_element_path(topic).ok_or(
        ModuleError::new().message("failed to get last element from mqtt topic".to_string()),
    )?;
    let module_ref_option = manager.connected_module.get_mut(id.as_str());
    if let Some(module_ref) = module_ref_option {
        module_ref.validator.remove_config(from_actor.clone())?;
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
    module_state_manager: &mut MainboardModuleStateManager,
    env_controller: &mut EnvControllerStore,
    data: Arc<Vec<u8>>,
) -> Result<(), MainboardError> {
    let field_alarm = FieldAlarm::parse_from_bytes(&data)?;
    alarm_store.add_alarm_field(&field_alarm)?;
    alarm_validator.register_field_alarm(field_alarm.clone(), None)?;

    env_controller.on_alarm_created(&field_alarm.moduleId, &field_alarm.property, module_state_manager, alarm_store, None, false)?;

    return Ok(());
}

fn handle_update_alarm(
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    module_state_manager: &mut MainboardModuleStateManager,
    env_controller: &mut EnvControllerStore,
    data: Arc<Vec<u8>>,
) -> Result<(), MainboardError> {
    let field_alarm = FieldAlarm::parse_from_bytes(&data)?;
    alarm_store.update_alarm_field(&field_alarm)?;
    alarm_validator.register_field_alarm(field_alarm.clone(), None)?;

    env_controller.on_alarm_deleted(&field_alarm.moduleId, &field_alarm.property, module_state_manager, alarm_store)?;
    env_controller.on_alarm_created(&field_alarm.moduleId, &field_alarm.property, module_state_manager, alarm_store, None, false)?;

    return Ok(());
}

fn handle_remove_alarm(
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    module_state_manager: &mut MainboardModuleStateManager,
    env_controller: &mut EnvControllerStore,
    data: Arc<Vec<u8>>,
) -> Result<(), MainboardError> {
    let field_alarm = FieldAlarm::parse_from_bytes(&data)?;
    alarm_store.remove_alarm_field(&field_alarm)?;
    alarm_validator.deregister_field_alarm(field_alarm.clone())?;

    env_controller.on_alarm_deleted(&field_alarm.moduleId, &field_alarm.property, module_state_manager, alarm_store)?;

    return Ok(());
}

fn handle_register_environment_controller(
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    module_state_manager: &mut MainboardModuleStateManager,
    env_controller: &mut EnvControllerStore,
    data: Arc<Vec<u8>>,
) -> Result<(), MainboardError> {
    println!("RECEIVE MAIN");
    let config = EnvironmentControllerConfiguration::parse_from_bytes(&data)?;

    env_controller.register_controller(module_state_manager, alarm_store, config)?;
    return Ok(());
}

fn handle_unregister_environment_controller(
    env_controller: &mut EnvControllerStore,
    topic: &String,
) -> Result<(), MainboardError> {
    let id = crate::utils::mqtt::last_element_path(topic).ok_or(
        MainboardError::new().message("failed to get last element from mqtt topic".to_string()),
    )?;

    return env_controller.unregister_controller(&id);
}



fn handle_validator_command(
    cmd: &str,
    module_id: &String,
    sender_response: tokio::sync::oneshot::Sender<crate::protos::message::ActionResponse>,
    manager: &mut MainboardModuleStateManager,
    sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    data: std::sync::Arc<Vec<u8>>,
    from_actor: &Actor,
) -> Result<std::option::Option<Vec<ModuleStateCmd>>, super::interface::ModuleError> {
    if let Some(module) = manager.connected_module.get_mut(module_id) {
        return module.validator.handle_command_validator(
            cmd,
            module_id,
            data,
            sender_response,
            sender_socket,
            from_actor.clone(),
        );
    } else {
        return Err(super::interface::ModuleError::not_found(module_id));
    }
}

pub fn handle_module_command(
    cmd: &String,
    topic: &String,
    actor: &Actor,
    data: std::sync::Arc<Vec<u8>>,
    sender_response: tokio::sync::oneshot::Sender<crate::protos::message::ActionResponse>,
    manager: &mut MainboardModuleStateManager,
    store: &super::store::ModuleStateStore,
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    sender_config: &ComboardSenderMapReference,
    sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    sender_module: &Sender<ModuleMsg>,
    virtual_relay_store: &mut super::relay::virtual_relay::store::VirtualRelayStore,
    mut env_controller: &mut EnvControllerStore,
) -> Result<(), MainboardError> {
    let result: Result<(), MainboardError> = match cmd.as_str() {
        "sync" => handle_sync_request(manager, &sender_socket),
        "pmconfig" => {
            handle_pmodule_config(topic, data, manager, &sender_config, &sender_socket, &store, &actor)
        },
        "mconfig" => {
            handle_module_config(topic, data, manager, &sender_config, &sender_socket, &store, &actor)
        }
        "rmconfig" => handle_remove_module_config(
            topic,
            data,
            manager,
            &sender_config,
            &sender_socket,
            &store,
            &actor,
        ),
        "aEnv" => handle_register_environment_controller(&alarm_store, manager, &mut env_controller, data),
        "rEnv" => handle_unregister_environment_controller(&mut env_controller, topic),
        "aAl" => handle_add_alarm(alarm_validator, &alarm_store, manager, &mut env_controller, data),
        "rAl" => handle_remove_alarm(alarm_validator, &alarm_store, manager, &mut env_controller, data),
        "uAl" => handle_update_alarm(alarm_validator, &alarm_store, manager, &mut env_controller, data),
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
            return match handle_validator_command(
                cmd,
                &module_id,
                sender_response,
                manager,
                &sender_socket,
                data,
                &actor,
            ) {
                Ok(option_cmd) => {
                    if let Some(cmds) = option_cmd {
                        for cmd in cmds {
                            sender_module.try_send(ModuleMsg::Cmd(cmd))?;
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(e.into()),
            };
        }
    };

    let mut action_respose = crate::protos::message::ActionResponse::new();
    match result {
        Ok(()) => {
            action_respose.status = 0;
            if let Err(err) = sender_response.send(action_respose) {
                log::error!("failed to send action response but cmd ok : {:?}", err);
            }
            println!("sending response for command");
            return Ok(());
        }
        Err(mainboard_error) => {
            action_respose.status = 500;
            action_respose.msg = mainboard_error.message.clone();
            if let Err(err) = sender_response.send(action_respose) {
                log::error!("failed to send action response : {:?}", err);
                log::error!("{:?}", mainboard_error);
            }
            println!("sending response error for command");
            return Err(mainboard_error);
        }
    }
}
