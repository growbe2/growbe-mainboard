#[cfg(test)]
pub mod tests {
    use crate::{
        comboard::imple::channel::ComboardSenderMapReference,
        modulestate::state_manager::MainboardConnectedModule,
    };

    use std::collections::HashMap;

    use tokio::sync::mpsc::{channel, Receiver, Sender};

    use crate::{
        comboard::imple::channel::{ComboardAddr, ComboardConfigChannelManager, ModuleConfig},
        modulestate::{
            alarm::{store::ModuleAlarmStore, validator::AlarmFieldValidator},
            controller::store::EnvControllerStore,
            interface::ModuleMsg,
            relay::virtual_relay::store::VirtualRelayStore,
            state_manager::MainboardModuleStateManager,
            store::ModuleStateStore,
        },
        socket::ss::SenderPayload,
    };

    lazy_static::lazy_static! {
        pub static ref AAP_MODULE_ID: String = "AAP0000003".into();
        pub static ref I2C_IMPL: String = "i2c".into();
        pub static ref I2C_ADDR: String = "/dev/i2c-1".into();
    }

    pub fn test_db(name: &str) -> std::sync::Arc<std::sync::Mutex<rusqlite::Connection>> {
        let conn = std::sync::Arc::new(std::sync::Mutex::new(crate::store::database::init(Some(
            format!("./database_test_{}.sqlite", name),
        ))));

        return conn;
    }

    pub fn init_context(
        test_name: &str,
    ) -> (
        tokio::sync::oneshot::Sender<crate::protos::message::ActionResponse>,
        tokio::sync::oneshot::Receiver<crate::protos::message::ActionResponse>,
        MainboardModuleStateManager,
        ModuleStateStore,
        AlarmFieldValidator,
        ModuleAlarmStore,
        Receiver<ModuleConfig>,
        ComboardSenderMapReference,
        Sender<SenderPayload>,
        Receiver<SenderPayload>,
        Sender<ModuleMsg>,
        Receiver<ModuleMsg>,
        VirtualRelayStore,
        EnvControllerStore,
    ) {
        let conn = test_db(test_name);

        let (s_ar, r_ar) = tokio::sync::oneshot::channel();

        let mut module_state_manager = MainboardModuleStateManager::new();
        let module_state_store = ModuleStateStore::new(conn.clone());
        let alarm_field_validator = AlarmFieldValidator::new();
        let module_alarm_store = ModuleAlarmStore::new(conn.clone());
        let mut comboard_sender = ComboardConfigChannelManager::new();

        let r_mc = comboard_sender.create_channel(ComboardAddr {
            imple: I2C_IMPL.clone(),
            addr: I2C_ADDR.clone(),
        });

        let (s_s, r_s) = channel(10);
        let (s_m, r_m) = channel(10);

        let virtual_relay_store = VirtualRelayStore::new(conn.clone());
        let env_controller_store =
            EnvControllerStore::new(conn.clone(), s_s.clone().into(), s_m.clone());

        module_state_manager.connected_module.insert(
            AAP_MODULE_ID.to_string(),
            MainboardConnectedModule {
                port: 0,
                id: AAP_MODULE_ID.clone(),
                board: I2C_IMPL.clone(),
                board_addr: I2C_ADDR.clone(),
                validator: crate::modulestate::modules::get_module_validator(&AAP_MODULE_ID[0..3])
                    .unwrap(),
                handler_map: HashMap::new(),
                last_value: None,
            },
        );

        return (
            s_ar,
            r_ar,
            module_state_manager,
            module_state_store,
            alarm_field_validator,
            module_alarm_store,
            r_mc,
            comboard_sender.get_reference(),
            s_s,
            r_s,
            s_m,
            r_m,
            virtual_relay_store,
            env_controller_store,
        );
    }
}
