use std::collections::HashMap;

use tokio_util::sync::CancellationToken;

use crate::{
    modulestate::alarm::model::ModuleValueChange,
    protos::{alarm::FieldAlarmEvent, env_controller::EnvironmentControllerConfiguration}, mainboardstate::error::MainboardError,
};

use super::module_command::ModuleCommandSender;

pub struct Context {
    pub cancellation_token: CancellationToken,
    // Sender for module config , to send a new config to apply and switch relay state.
    pub module_command_sender: ModuleCommandSender,

    // Receiver alarm , to receive alarm
    pub alarm_receivers: HashMap<String, tokio::sync::watch::Receiver<FieldAlarmEvent>>,

    // Receiver value change, map with a watch receiver for each module
    pub value_receivers: HashMap<String, tokio::sync::watch::Receiver<ModuleValueChange<f32>>>,
}

/*
 * EnvControllerTask is the trait for all implement
 * of environment controller task
 * */
pub trait EnvControllerTask {
    fn run(
        &self,
        config: EnvironmentControllerConfiguration,
        context: Context,
    ) -> Result<tokio::task::JoinHandle<Result<(), MainboardError>>, MainboardError>;
}
