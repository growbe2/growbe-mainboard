
use protobuf::Message;

use crate::modulestate::relay::virtual_relay::op::initialize_virtual_relay_and_apply_config;

use super::{store::{VirtualRelayStore}, op::{is_virtual_relay_required_module, initialize_virtual_relay, apply_config_virtual_relay, delete_virtual_relay}};

pub fn on_module_state_changed_virtual_relays(
    state: bool,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(), ()> {
    let config_relays = store_virtual_relay.get_stored_relays().unwrap();
    let connected_modules = manager.get_connected_modules();

    if state {
        // regarde si je dois demarrer des virtual relays
        for (vr, opt_config) in config_relays {
            // valide si j'existe deja first
            if !store_virtual_relay.is_created(vr.get_name()) {
                if is_virtual_relay_required_module(&connected_modules, &vr) {
                    log::info!("creating virtual relay {}", vr.get_name());
                    initialize_virtual_relay_and_apply_config(&vr, &opt_config, sender_comboard_config, sender_socket, store, store_virtual_relay, manager);

                    let mut state = crate::protos::module::VirtualRelayState::new();
                    state.set_id(vr.get_name().to_string());
                    state.set_state(true);
                    sender_socket.send((format!("/vr/{}/vrstate", vr.get_name()), Box::new(state))).unwrap();
                } else {
                    // cant create the vr missing modules
                }
            } else {
                // already created do nothing
            }
        }
        
    } else {
        // Je dois valider si je dois desactiver des virtuals relays
        for (vr, _opt_config) in config_relays {
            if store_virtual_relay.is_created(vr.get_name()) {
                if !is_virtual_relay_required_module(&connected_modules, &vr) {
                    log::info!("deleting virtual relay {}", vr.get_name());
                    store_virtual_relay.stop_virtual_relay(vr.get_name());
                    let mut state = crate::protos::module::VirtualRelayState::new();
                    state.set_id(vr.get_name().to_string());
                    state.set_state(false);
                    sender_socket.send((format!("/vr/{}/vrstate", vr.get_name()), Box::new(state))).unwrap();
                }
            }
        }
    }

    return Ok(());
}



// HANDLING FUNCTION FOR ROUTER

// handle the creating and destruction of virtual relay
// do this everytime a module connect or disconnect because
// it may affect the virtual relay, cannot create
// one if 
pub fn handle_virtual_relay(
    data: std::sync::Arc<Vec<u8>>,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(),()> {

    let relay_config = crate::protos::module::VirtualRelay::parse_from_bytes(&data).unwrap();

    return initialize_virtual_relay(&relay_config, sender_comboard_config, sender_socket, store, store_virtual_relay, manager)
}

pub fn handle_apply_config_virtual_relay(
    topic: &String,
    data: std::sync::Arc<Vec<u8>>,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(), ()> {

    let id = crate::utils::mqtt::last_element_path(topic);

    let config = crate::protos::module::RelayOutletConfig::parse_from_bytes(&data).unwrap();

    return apply_config_virtual_relay(&id, &config, sender_comboard_config, sender_socket, store, store_virtual_relay, manager);
}

pub fn handle_delete_virtual_relay(
    topic: &String,
    _data: std::sync::Arc<Vec<u8>>,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(), ()> {

    let id = crate::utils::mqtt::last_element_path(topic);

    return delete_virtual_relay(
        &id,
        sender_comboard_config,
        sender_socket,
        store,
        store_virtual_relay,
        manager
    );
}
