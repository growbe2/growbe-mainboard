use crate::modulestate::interface::{ModuleValue, ModuleValueParsable};
use crate::modulestate::relay::{Relay, State};
use crate::protos::module::{RelayOutletData, VirtualRelayData, VirtualRelayState};

impl ModuleValue for VirtualRelayData {}
impl ModuleValueParsable for VirtualRelayData {}
impl ModuleValue for VirtualRelayState {}
impl ModuleValueParsable for VirtualRelayState {}

impl ModuleValue for RelayOutletData {}
impl ModuleValueParsable for RelayOutletData {}

// Fake relay that control mutliple physical relay to all triger them
// together in a group
pub struct VirtualRelay {
    pub name: String,
    pub sender_socket: tokio::sync::mpsc::Sender<(
        String,
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    )>,
    pub relays: Vec<Box<dyn Relay>>,
}

impl VirtualRelay {
    pub fn new(
        name: &str,
        sender_socket: &tokio::sync::mpsc::Sender<(
            String,
            Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        )>,
    ) -> Self {
        return VirtualRelay {
            name: name.to_string(),
            relays: vec![],
            sender_socket: sender_socket.clone(),
        };
    }
}

impl State for VirtualRelay {
    fn set_state(&mut self, state: u8) -> Result<(), ()> {
        self.relays
            .iter_mut()
            .for_each(|x| x.set_state(state).unwrap());

        let mut vr_data = VirtualRelayData::new();
        let mut data = RelayOutletData::new();
        data.set_state(state != 0);

        vr_data.set_data(data);
        vr_data.set_timestamp(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32,
        );
        self.sender_socket
            .try_send((format!("/vr/{}/vrdata", self.name), Box::new(vr_data))).unwrap();
        return Ok(());
    }
}

impl Relay for VirtualRelay {
    fn id(&self) -> String {
        return self.name.clone();
    }
    fn clone_me(&self) -> Box<dyn Relay> {
        let relays: Vec<Box<dyn Relay>> = self
            .relays
            .iter()
            .map(|x| {
                return x.clone_me();
            })
            .collect();
        return Box::new(VirtualRelay {
            name: self.name.clone(),
            sender_socket: self.sender_socket.clone(),
            relays: relays,
        });
    }
}
