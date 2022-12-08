use tokio::select;
use crate::{modulestate::controller::controller_trait::{EnvControllerTask, Context}, protos::env_controller::EnvironmentControllerConfiguration, mainboardstate::error::MainboardError};


pub struct StaticControllerImplementation {}


impl StaticControllerImplementation {
    pub fn new() -> Self {
        Self {}
    }
}

impl EnvControllerTask for StaticControllerImplementation {
    fn run(
        &self,
        config: EnvironmentControllerConfiguration,
        context: Context,
    ) -> Result<tokio::task::JoinHandle<Result<(), MainboardError>>, MainboardError> {
        return Ok(tokio::task::spawn(async move {
            log::info!("starting static controller : {}", config.get_id());
            loop {
                select! {
                    _ = context.cancellation_token.cancelled() => {
                        log::info!("static controller stopped");
                        return Ok(());
                    }
                }
            }

        }));
    }
}
