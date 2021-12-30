
use tokio_util::sync::CancellationToken;
use super::{physical_relay::{PhysicalRelay, BatchPhysicalRelay, ActionPortUnion}, Relay};

// Fake relay that control mutliple physical relay to all triger them
// together in a group
pub struct VirtualRelay {
    pub name: String,
    pub relays: Vec<Box<dyn Relay>>
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
    relay_config: &crate::protos::module::VirtualOutlet,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    manager: & crate::modulestate::MainboardModuleStateManager,
) -> Result<VirtualRelay, ()> {
    
    let mut virtual_relay = VirtualRelay::new(relay_config.get_name());

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
                action_port: (*v.properties.get(0).unwrap()) as usize,
            });
            virtual_relay.relays.push(relay);
        } else {
            let batch_relay = Box::new(BatchPhysicalRelay{
                action_port: ActionPortUnion::new_ports(v.properties.iter().map(|x| (*x) as usize).collect()),
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

// handle the creating and destruction of virtual relay
// do this everytime a module connect or disconnect because
// it may affect the virtual relay, cannot create
// one if 
pub fn handle_virtual_relay(
    relay_config: &crate::protos::module::VirtualOutlet,
    virtual_relay_maps: &mut std::collections::HashMap<String, VirtualRelay>,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: crate::modulestate::store::ModuleStateStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(),()> {

    // check if im already existing , if not , delete me and recreate me ??
    if virtual_relay_maps.contains_key(relay_config.get_name()) {
        match delete_virtual_relay(
            relay_config.get_name(),
            virtual_relay_maps,
            sender_comboard_config,
            sender_socket,
            store,
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

    let relay = create_virtual_relay(relay_config, sender_comboard_config, manager)?;

    virtual_relay_maps.insert(relay_config.get_name().to_string(), relay);

    return Ok(());
}

pub fn apply_config_virtual_relay(
    virtual_relay_maps: &mut std::collections::HashMap<String, VirtualRelay>,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: crate::modulestate::store::ModuleStateStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
    map_handler: & mut std::collections::HashMap<String, CancellationToken>,
) -> Result<(), ()> {



    return Ok(());
}

pub fn delete_virtual_relay(
    name: &str,
    virtual_relay_maps: &mut std::collections::HashMap<String, VirtualRelay>,
    sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    sender_socket: std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: crate::modulestate::store::ModuleStateStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(), ()> {

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
