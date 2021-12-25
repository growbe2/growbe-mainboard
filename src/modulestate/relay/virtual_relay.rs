
use tokio_util::sync::CancellationToken;

// Fake relay that control mutliple physical relay to all triger them
// together in a group
pub struct VirtualRelay {
    pub name: String,
    pub relays: Vec<super::physical_relay::PhysicalRelay>
}

pub struct BatchVirtualRelay {
    pub relays: Vec<super::physical_relay::PhysicalRelay>
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
    fn clone(&self) -> Box<dyn super::Relay> {
        let relays: Vec<super::physical_relay::PhysicalRelay> = self.relays.iter()
            .map(|x| super::physical_relay::PhysicalRelay { sender: x.sender.clone(), port: x.port, action_port: x.action_port })
            .collect();
        return Box::new(VirtualRelay{
            name: self.name.clone(),
            relays: relays,
        });
    }
}



fn create_virtual_relay(
    relay: &crate::protos::module::VirtualOutlet,
    virtual_relay_maps: &mut std::collections::HashMap<String, VirtualRelay>,
    sender_socket: std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: crate::modulestate::store::ModuleStateStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<VirtualRelay, ()> {


    return Err(());
}

// handle the creating and destruction of virtual relay
// do this everytime a module connect or disconnect because
// it may affect the virtual relay, cannot create
// one if 
pub fn handle_virtual_relay(
    virtual_relay_maps: &mut std::collections::HashMap<String, VirtualRelay>,
    sender_socket: std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: crate::modulestate::store::ModuleStateStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
) -> Result<(),()> {
    // create the virtual relay

    // delete the previous one

    return Ok(());
}

pub fn apply_config_virtual_relay(
    virtual_relay_maps: &mut std::collections::HashMap<String, VirtualRelay>,
    sender_socket: std::sync::mpsc::Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    store: crate::modulestate::store::ModuleStateStore,
    manager: & mut crate::modulestate::MainboardModuleStateManager,
    map_handler: & mut std::collections::HashMap<String, CancellationToken>,
) -> Result<(), ()> {



    return Ok(());
}

// BatchRelay n'est pas vrnécessaire pour le moment pour relais virtuelle
// car c'est dequoi qui est seulement utilisé lors application config pour enviter
// de faire plusieurs request.
// Serait surement utile lorsqu'on va initialiser plusieurs relais virtuelle
/*
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
