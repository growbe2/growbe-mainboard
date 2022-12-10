use crate::{
    mainboardstate::error::MainboardError,
    modulestate::{
        cmd::CHANNEL_MODULE_STATE_CMD,
        interface::{ModuleStateCmd, ModuleValueParsable},
    },
    protos::{message::ActionResponse, module::RelayOutletConfig},
};
use std::sync::mpsc::Receiver;

pub struct ModuleCommandSender {}

impl ModuleCommandSender {
    pub fn new() -> Self {
        Self {}
    }

    pub fn send_relay_config(
        &self,
        type_relay: &str,
        id: &str,
        config: RelayOutletConfig,
    ) -> Result<Receiver<ActionResponse>, MainboardError> {
        return Err(MainboardError::new());
    }

    pub fn send_vrconfig(
        &self,
        id: &str,
        config: Box<dyn ModuleValueParsable>,
    ) -> Result<Receiver<ActionResponse>, MainboardError> {
        return self.send_cmd("vrconfig", id, config);
    }

    pub fn send_mconfig(
        &self,
        id: &str,
        config: Box<dyn ModuleValueParsable>,
    ) -> Result<Receiver<ActionResponse>, MainboardError> {
        return self.send_cmd("mconfig", id, config);
    }

    pub fn send_mconfig_prop(
        &self,
        id: &str,
        prop: &str,
        config: Box<dyn ModuleValueParsable>,
    ) -> Result<Receiver<ActionResponse>, MainboardError> {
        return self.send_cmd("pmconfig", &format!("{}/{}", id, prop), config);
    }

    fn send_cmd(
        &self,
        cmd: &'static str,
        id: &str,
        config: Box<dyn ModuleValueParsable>,
    ) -> Result<std::sync::mpsc::Receiver<ActionResponse>, MainboardError> {
        let (sender, receiver) = std::sync::mpsc::channel::<ActionResponse>();
        let cmd = ModuleStateCmd {
            cmd,
            topic: format!("local:/{}", id),
            sender,
            data: std::sync::Arc::new(config.write_to_bytes()?),
        };
        CHANNEL_MODULE_STATE_CMD
            .0
            .lock()
            .map_err(|x| MainboardError::from_error(x.to_string()))?
            .send(cmd)
            .map_err(|x| MainboardError::from_error(x.to_string()))?;
        return Ok(receiver);
    }
}
