use tokio_util::sync::CancellationToken;

use crate::modulestate::relay::{physical_relay::{PhysicalRelay, BatchPhysicalRelay, ActionPortUnion}, configure::configure_relay};

use super::{store::VirtualRelayStore, virtual_relay::VirtualRelay};


pub fn create_virtual_relay(
    relay_config: &crate::protos::module::VirtualRelay,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    manager: & crate::modulestate::MainboardModuleStateManager,
    store_virtual_relay: & mut VirtualRelayStore,
) -> Result<VirtualRelay, ()> {
    
    let mut virtual_relay = VirtualRelay::new(relay_config.get_name(), sender_socket);

    store_virtual_relay.store_relay(relay_config).unwrap();

    for (k, v) in relay_config.get_relays().iter() {

        let module_ref_options = manager.connected_module.get(k);

        if module_ref_options.is_none() {
            return Err(());
        }

        let module_ref = module_ref_options.unwrap();

        // if only one propertie use a normal relay
        if v.properties.len() == 1 {
            let relay: Box<PhysicalRelay> = Box::new(PhysicalRelay{
                sender: sender_comboard_config.clone(),
                port: module_ref.port,
                action_port: (*v.properties.get(0).unwrap()).property as usize,
            });
            virtual_relay.relays.push(relay);
        } else {
            let batch_relay = Box::new(BatchPhysicalRelay{
                action_port: ActionPortUnion::new_ports(v.properties.iter().map(|x| (*x).property as usize).collect()),
                buffer: [255; 8],
                auto_send: true,
                port: module_ref.port,
                sender: sender_comboard_config.clone(),
            });
            virtual_relay.relays.push(batch_relay);
        }
    }


    return Ok(virtual_relay);
}

pub fn delete_virtual_relay(
    name: &str,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(), ()> {


    if store_virtual_relay.is_created(name)  {
        store_virtual_relay.stop_virtual_relay(name);
        let mut state = crate::protos::module::VirtualRelayState::new();
        state.set_id(name.to_string());
        state.set_state(false);
        sender_socket.send((format!("/vr/{}/vrstate", name), Box::new(state))).unwrap();
    }

    store_virtual_relay.remove_relay(name);

    log::info!("virtual relay deleted {}", name);
    return Ok(());
}

pub fn initialize_virtual_relay(
    relay_config: &crate::protos::module::VirtualRelay, 
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(),()> {


    // check if im already existing , if not , delete me and recreate me ??
    if store_virtual_relay.virtual_relay_maps.contains_key(relay_config.get_name()) {
        match delete_virtual_relay(
            relay_config.get_name(),
            sender_comboard_config,
            sender_socket,
            store,
            store_virtual_relay,
            manager,
        ) {
            Ok(()) => {
                log::debug!("deleting virtual relay {}", relay_config.get_name());
            },
            Err(()) => {
                // error cannot delete existing one
                return Err(());
            },
        }
    }

    let relay = create_virtual_relay(&relay_config, sender_socket, sender_comboard_config, manager, store_virtual_relay)?;

    let clone_str = relay.name.clone();
    store_virtual_relay.virtual_relay_maps.insert(relay.name.clone(),VirtualRelay { name: clone_str, relays: relay.relays, sender_socket: relay.sender_socket });
    store_virtual_relay.cancellation_token_maps.insert(relay.name.clone(), CancellationToken::new());

    let mut state = crate::protos::module::VirtualRelayState::new();
    state.set_id(relay.name.clone());
    state.set_state(true);
    sender_socket.send((format!("/vr/{}/vrstate", state.get_id()), Box::new(state))).unwrap();

    return Ok(());
}

pub fn apply_config_virtual_relay(
    id: &String,
    config: &crate::protos::module::RelayOutletConfig,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(), ()> {

    match store_virtual_relay.virtual_relay_maps.get_mut(id) {
        Some(relay) => {
            configure_relay(true, &config, relay, & mut store_virtual_relay.cancellation_token_maps, None);
            // HACK , fixe issue of task not starting for relay ?????
            configure_relay(true, &config, relay, & mut store_virtual_relay.cancellation_token_maps, None);
            store_virtual_relay.store_relay_config(id, config).unwrap();
            return Ok(());
        },
        None => return Err(()),
    }
}

pub fn is_virtual_relay_required_module(
    modules: &Vec<String>,
    virtual_relay: &crate::protos::module::VirtualRelay,
) -> bool {
    return virtual_relay.get_relays().keys().all(|e| modules.contains(e));
}

pub fn initialize_virtual_relay_and_apply_config(
    virtual_relay: &crate::protos::module::VirtualRelay, 
    virtual_config: &Option<crate::protos::module::RelayOutletConfig>,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) {
    initialize_virtual_relay(&virtual_relay, sender_comboard_config, sender_socket, store, store_virtual_relay, manager).unwrap();
    if let Some(config) = virtual_config.as_ref() {
        apply_config_virtual_relay(&String::from(virtual_relay.get_name()), config, sender_comboard_config, sender_socket, store, store_virtual_relay, manager).unwrap();
    }
}
