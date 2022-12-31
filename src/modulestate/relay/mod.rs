pub mod alarm;
pub mod configure;
pub mod cycle;
pub mod duration;
pub mod physical_relay;
pub mod virtual_relay;

use crate::protos::module::RelayOutletData;
use protobuf::SingularPtrField;

fn f(i: &usize, x: &mut [u8], value: u8) {
    x[*i] = value;
}

pub fn get_outlet_data(value: u8) -> SingularPtrField<RelayOutletData> {
    let mut data = RelayOutletData::new();
    if value == 0 {
        data.set_state(false);
    } else if value == 1 {
        data.set_state(true);
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
                $this.$name = SingularPtrField::from(Some($data.clone()));
            }
        )+
    };
}
