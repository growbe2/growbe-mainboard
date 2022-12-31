use crate::{
    mainboardstate::error::MainboardError, protos::env_controller::EnvironmentControllerEvent,
};

/*
 * EnvControllerTask is the trait for all implement
 * of environment controller task
 * */
pub trait EnvControllerTask {
    fn run(
        &self,
        context: super::context::Context,
    ) -> Result<tokio::task::JoinHandle<Result<(), MainboardError>>, MainboardError>;
}

impl crate::modulestate::interface::ModuleValue for EnvironmentControllerEvent {}
impl crate::modulestate::interface::ModuleValueParsable for EnvironmentControllerEvent {}
