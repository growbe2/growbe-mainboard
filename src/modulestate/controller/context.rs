
use std::collections::HashMap;

use tokio_util::sync::CancellationToken;

use crate::{
    modulestate::alarm::model::ModuleValueChange,
    protos::{alarm::FieldAlarmEvent, env_controller::{EnvironmentControllerConfiguration, EnvironmentControllerEvent, EnvironmentControllerState, MActor, MObserver}},socket::ss::SenderSocket,
};

use super::module_command::ModuleCommandSender;

pub struct Context {
    pub config: EnvironmentControllerConfiguration,

    pub cancellation_token: CancellationToken,
    // Sender for module config , to send a new config to apply and switch relay state.
    pub module_command_sender: ModuleCommandSender,
    // To send message to the socket
    pub sender_socket: SenderSocket,
    // Receiver alarm , to receive alarm
    pub alarm_receivers: HashMap<String, tokio::sync::watch::Receiver<FieldAlarmEvent>>,

    // Receiver value change, map with a watch receiver for each module
    pub value_receivers: HashMap<String, tokio::sync::watch::Receiver<ModuleValueChange<f32>>>,
}


#[macro_export]
macro_rules! send_event {
    ($self: ident, $state: expr, $running: ident) => {
        let mut event = EnvironmentControllerEvent::new();
        event.id = $self.config.id.clone();
        event.running = $running;
        event.state = $state;
        if let Err(err) = $self.sender_socket.send(format!("/env_ctr/{}", $self.config.id), Box::new(event)) {
            log::error!("{:?}", err);
        }
       
    };
}

#[macro_export]
macro_rules! get_env_element {
    ($self: ident, $element: ident, $value: ident) => {
       $self.config
            .$element
            .iter()
            .find(|x| x.name.eq($value))
    };
}


