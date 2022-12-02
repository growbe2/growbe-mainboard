use std::sync::mpsc::Sender;
use std::collections::HashMap;

use crate::comboard::imple::channel::{ComboardSenderMapReference, CHANNEL_STATE, CHANNEL_VALUE};
use super::state_manager::MainboardModuleStateManager;

pub fn module_state_task(
    sender_socket: Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    store: super::store::ModuleStateStore,
    sender_config: ComboardSenderMapReference,
    alarm_store: super::alarm::store::ModuleAlarmStore,
) -> tokio::task::JoinHandle<()> {
    return tokio::spawn(async move {
        let mut manager = MainboardModuleStateManager {
            connected_module: HashMap::new(),
        };

        let mut alarm_validator = super::alarm::validator::AlarmFieldValidator::new();

        let mut virtual_relay_store =
            super::relay::virtual_relay::store::VirtualRelayStore::new(alarm_store.conn.clone());

        let receiver_state = CHANNEL_STATE.1.lock().unwrap();
        let receiver_value = CHANNEL_VALUE.1.lock().unwrap();

        loop {
            {
                if let Ok(mut state) = receiver_state.try_recv() {
                    if let Err(err) = super::handle_state::handle_module_state(
                        &mut manager,
                        &mut state,
                        &sender_config,
                        &sender_socket,
                        &store,
                        &mut alarm_validator,
                        &alarm_store,
                    ) {
                        log::error!("failed to handle_modle_state : {:?}", err);
                    }
                    if let Err(()) = super::relay::virtual_relay::handler::on_module_state_changed_virtual_relays(
                        state.state,
                        &sender_config,
                        &sender_socket,
                        &store,
                        &mut virtual_relay_store,
                        &mut manager,
                    ) {
                        log::error!("failed to changed virtual_relay state");
                    }
                }
            }
            {
                if let Ok(value) = receiver_value.try_recv() {
                    if let Err(err) = super::handle_value::handle_module_value(
                        &mut manager,
                        &value,
                        &sender_socket,
                        &mut alarm_validator,
                        &alarm_store,
                    ) {
                        log::error!("failed to handle_module_value : {:?}", err);
                    }
                }
            }
            {
                if let Ok(cmd) = super::cmd::CHANNEL_MODULE_STATE_CMD.1.lock().unwrap().try_recv() {
                    if let Err(err) = super::cmd::handle_module_command(
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
                    ) {
                        log::error!("failed handle_module_command : {:?}", err);
                    }
                }
            }
        }
    });
}
