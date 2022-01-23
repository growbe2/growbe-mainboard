
use crate::modulestate::interface::{ModuleValue, ModuleValueParsable};
use crate::modulestate::relay::{Relay, State};
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


impl VirtualRelay {
    pub fn new(name: &str) -> Self {
        return VirtualRelay { name: name.to_string(), relays: vec![]}
    }
}

impl State for VirtualRelay {
    fn set_state(&mut self, state: u8) -> Result<(), ()> {
        self.relays.iter_mut().for_each(|x| x.set_state(state).unwrap());
        return Ok(());
    }
}

impl Relay for VirtualRelay {
    fn id(&self) -> String {
        return self.name.clone();
    }
    fn clone_me(&self) -> Box<dyn Relay> {
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

