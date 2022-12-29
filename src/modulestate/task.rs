use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::socket::ss::SenderPayload;

use super::{
    controller::store::EnvControllerStore, interface::ModuleMsg,
    state_manager::MainboardModuleStateManager,
};
use crate::comboard::imple::channel::ComboardSenderMapReference;

pub fn module_state_task(
    sender_socket: Sender<SenderPayload>,
    store: super::store::ModuleStateStore,
    sender_config: ComboardSenderMapReference,
    alarm_store: super::alarm::store::ModuleAlarmStore,

    sender: tokio::sync::mpsc::Sender<ModuleMsg>,
    mut receiver: Receiver<ModuleMsg>,
) -> tokio::task::JoinHandle<()> {
    return tokio::spawn(async move {
        let mut manager = MainboardModuleStateManager {
            connected_module: HashMap::new(),
        };

        let mut alarm_validator = super::alarm::validator::AlarmFieldValidator::new();

        let mut virtual_relay_store =
            super::relay::virtual_relay::store::VirtualRelayStore::new(alarm_store.conn.clone());

        let mut environment_controller = EnvControllerStore::new(
            alarm_store.conn.clone(),
            sender_socket.clone().into(),
            sender.clone(),
        );

        loop {
            if let Some(msg) = receiver.recv().await {
                match msg {
                    ModuleMsg::Cmd(cmd) => {
                        if let Err(err) = super::cmd::handle_module_command(
                            &cmd.cmd,
                            &cmd.topic,
                            &cmd.actor,
                            cmd.data,
                            cmd.sender,
                            &mut manager,
                            &store,
                            &mut alarm_validator,
                            &alarm_store,
                            &sender_config,
                            &sender_socket,
                            &sender,
                            &mut virtual_relay_store,
                            &mut environment_controller,
                        ) {
                            log::error!("failed handle_module_command : {:?}", err);
                        }
                    }
                    ModuleMsg::State(state) => {
                        if let Err(err) = super::handle_state::handle_module_state(
                            &mut manager,
                            &state,
                            &sender_config,
                            &sender_socket,
                            &store,
                            &mut alarm_validator,
                            &alarm_store,
                            &mut environment_controller,
                        ) {
                            log::error!("failed to handle_modle_state : {:?}", err);
                        }

                        if let Err(err) =
                        super::relay::virtual_relay::handler::on_module_state_changed_virtual_relays(
                            state.state,
                            &sender_config,
                            &sender_socket,
                            &store,
                            &mut virtual_relay_store,
                            &mut manager,
                        )
                    {
                        log::error!("failed to changed virtual_relay state {:#?}", err);
                    }
                    }
                    ModuleMsg::Value(value) => {
                        if let Err(err) = super::handle_value::handle_module_value(
                            &mut manager,
                            &value,
                            &sender_socket,
                            &mut alarm_validator,
                            &alarm_store,
                            &mut environment_controller,
                        ) {
                            log::error!("failed to handle_module_value : {:?}", err);
                        }
                    }
                }
            }
        }
    });
}
