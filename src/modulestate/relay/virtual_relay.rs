
use std::sync::{Arc, Mutex};

use super::{physical_relay::{PhysicalRelay, BatchPhysicalRelay, ActionPortUnion}, Relay};

use protobuf::Message;

use tokio_util::sync::CancellationToken;

use crate::modulestate::interface::{ModuleValue, ModuleValueParsable};
use crate::protos::module::{VirtualRelayData, VirtualRelayState};

impl ModuleValue for VirtualRelayData {}

impl ModuleValueParsable for VirtualRelayData {}

impl ModuleValue for VirtualRelayState {}

impl ModuleValueParsable for VirtualRelayState {}

// Fake relay that control mutliple physical relay to all triger them
// together in a group
pub struct VirtualRelay {
    pub name: String,
    pub relays: Vec<Box<dyn Relay>>
}

pub struct VirtualRelayStore {
    pub conn: Arc<Mutex<rusqlite::Connection>>,
    pub virtual_relay_maps: std::collections::HashMap<String, VirtualRelay>,
    pub cancellation_token_maps: std::collections::HashMap<String, CancellationToken>,
}

impl VirtualRelayStore {
    pub fn new(
        conn: Arc<Mutex<rusqlite::Connection>>,
    ) -> Self {
        VirtualRelayStore {
            conn: conn,
            virtual_relay_maps: std::collections::HashMap::new(),
            cancellation_token_maps: std::collections::HashMap::new(),
        }
    }

    pub fn is_created(&self, virtual_relay_id: &str) -> bool {
        return self.virtual_relay_maps.contains_key(virtual_relay_id);
    }

    pub fn stop_virtual_relay(&mut self, id: &str) {
        let d = self.virtual_relay_maps.remove(id);
        if d.is_some() {
            if let Some(cancellation_token) = self.cancellation_token_maps.remove(id) {
                cancellation_token.cancel();
            }
        }
    }

    pub fn store_relay(&self, config: &crate::protos::module::VirtualRelay) -> Result<(), ()> {
        crate::store::database::store_field_from_table(&self.conn, "virtual_relay", &String::from(config.get_name()), "relay", Box::new(config.clone()));
        return Ok(());
    }

    pub fn remove_relay(&self, id: &str) {
        // remove relay and its configj


        self.remove_relay_config().unwrap();
    }

    pub fn get_stored_relays(&self,) -> Result<Vec<(crate::protos::module::VirtualRelay, Option<crate::protos::module::RelayOutletConfig>)>, ()> {
        return crate::store::database::get_fields_from_table(
            &self.conn, "virtual_relay", "relay", "config",
        crate::protos::module::VirtualRelay::parse_from_bytes, crate::protos::module::RelayOutletConfig::parse_from_bytes
        ).map_err(|x| ());
    }

    pub fn remove_relay_config(&self,) -> Result<(), ()> {


        return Ok(());
    }

    pub fn store_relay_config(&self,) {

    }

}

impl VirtualRelay {
    pub fn new(name: &str) -> Self {
        return VirtualRelay { name: name.to_string(), relays: vec![]}
    }
}


impl super::State for VirtualRelay {
    fn set_state(&mut self, state: u8) -> Result<(), ()> {
        self.relays.iter_mut().for_each(|x| x.set_state(state).unwrap());
        return Ok(());
    }
}

impl super::Relay for VirtualRelay {
    fn id(&self) -> String {
        return self.name.clone();
    }
    fn clone_me(&self) -> Box<dyn super::Relay> {
        let relays: Vec<Box<dyn Relay>> = self.relays.iter()
            .map(|x| {
                return x.clone_me();
            })
            .collect();
        return Box::new(VirtualRelay{
            name: self.name.clone(),
            relays: relays,
        });
    }
}



fn create_virtual_relay(
    relay_config: &crate::protos::module::VirtualRelay,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    manager: & crate::modulestate::MainboardModuleStateManager,
    store_virtual_relay: & mut VirtualRelayStore,
) -> Result<VirtualRelay, ()> {
    
    let mut virtual_relay = VirtualRelay::new(relay_config.get_name());

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



fn initialize_virtual_relay(
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

    let relay = create_virtual_relay(&relay_config, sender_comboard_config, manager, store_virtual_relay)?;

    println!("VR: {:?}", relay.relays.len());

    let clone_str = relay.name.clone();
    store_virtual_relay.virtual_relay_maps.insert(relay.name,VirtualRelay { name: clone_str, relays: relay.relays });

    return Ok(());
}

fn apply_config_virtual_relay(
    id: &String,
    config: &crate::protos::module::RelayOutletConfig,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: & crate::modulestate::store::ModuleStateStore,
    store_virtual_relay: & mut VirtualRelayStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(), ()> {

    println!("Config {:?}", config);

    match store_virtual_relay.virtual_relay_maps.get_mut(id) {
        Some(relay) => {
            super::configure::configure_relay(true, &config, relay, & mut store_virtual_relay.cancellation_token_maps, None);
            return Ok(());
        },
        None => return Err(()),
    }
}

fn is_virtual_relay_required_module(
    modules: &Vec<String>,
    virtual_relay: &crate::protos::module::VirtualRelay,
) -> bool {
    return virtual_relay.get_relays().keys().all(|e| modules.contains(e));
}

fn initialize_virtual_relay_and_apply_config(
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
    return Ok(());
}

// BatchRelay n'est pas vrnécessaire pour le moment pour relais virtuelle
// car c'est dequoi qui est seulement utilisé lors application config pour enviter
// de faire plusieurs request.
// Serait surement utile lorsqu'on va initialiser plusieurs relais virtuelle
/*

pub struct BatchVirtualRelay {
    pub relays: Vec<super::physical_relay::PhysicalRelay>
}


impl super::State for BatchVirtualRelay {
    fn set_state(&mut self, state: u8) -> Result<(), ()> {

        return Ok(());
    }
}

impl super::Relay for BatchVirtualRelay {
    fn id(&self) -> String {
        return format!("{}", self.action_port);
    }

    fn clone(&self) -> Box<dyn super::Relay> {
        return Box::new(PhysicalRelay{
            sender: self.sender.clone(),
            port: self.port,
            action_port: self.action_port,
        });
    }
}

impl super::BatchRelay for BatchVirtualRelay {
    fn execute(&self) -> Result<(),()> {
        return Ok(());
    }
}
*/



#[cfg(test)]
mod tests {

    use super::*;
    use crate::protos::module::VirtualRelay;

    

}
