pub mod alarm;
pub mod configure;
pub mod cycle;
pub mod duration;
pub mod physical_relay;
pub mod virtual_relay;

use crate::{protos::module::RelayOutletData, utils::time::get_timestamp};
use protobuf::SingularPtrField;

fn f(i: &usize, x: &mut [u8], value: u8) {
    x[*i] = value;
}

pub fn get_outlet_data(value: u8, previous_value: &RelayOutletData) -> SingularPtrField<RelayOutletData> {
    let mut data = RelayOutletData::new();
    if value == 0 {
        data.set_state(false);
    } else if value == 1 {
        data.set_state(true);
    }
    if data.get_state() != previous_value.get_state() {
        data.set_timestamp(get_timestamp());
    } else {
        data.set_timestamp(previous_value.get_timestamp());
    }
    return SingularPtrField::some(data);
}

pub trait State {
    fn set_state(&mut self, state: u8) -> Result<(), ()>;
}

pub trait Relay: State + Send {
    fn id(&self) -> String;
    fn clone_me(&self) -> Box<dyn Relay>;
}

pub trait BatchRelay: Relay {
    fn execute(&self) -> Result<(), ()>;
}

#[macro_export]
macro_rules! set_property {
    ($this: ident, $property: ident, $data: ident, $($name: ident),+) => {
        $(
            if $property == stringify!($name) {
                if let Some(d) = $this.$name.into_option() {
                    $data.actor_owner_id = d.actor_owner_id.clone();
                    $data.actor_owner_type = d.actor_owner_type;
                }
                $data.timestamp = crate::utils::time::get_timestamp();
                $this.$name = SingularPtrField::from(Some($data.clone()));
            }
        )+
    };
}
