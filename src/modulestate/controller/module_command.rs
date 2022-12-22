use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

use crate::{
    mainboardstate::error::MainboardError,
    modulestate::interface::{ModuleStateCmd, ModuleValueParsable, ModuleMsg},
    protos::{message::ActionResponse, module::{RelayOutletConfig, Actor}},
};

pub struct ModuleCommandSender {
    sender_module: Sender<ModuleMsg>,
}

impl ModuleCommandSender {
    pub fn new(sender_module: Sender<ModuleMsg>) -> Self {
        Self {
            sender_module,
        }
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
        actor: Actor,
    ) -> Result<Receiver<ActionResponse>, MainboardError> {
        return self.send_cmd("vrconfig", id, config, actor);
    }

    pub fn send_mconfig(
        &self,
        id: &str,
        config: Box<dyn ModuleValueParsable>,
        actor: Actor,
    ) -> Result<Receiver<ActionResponse>, MainboardError> {
        return self.send_cmd("mconfig", id, config, actor);
    }

    pub fn send_mconfig_prop(
        &self,
        id: &str,
        prop: &str,
        config: Box<dyn ModuleValueParsable>,
        actor: Actor,
    ) -> Result<Receiver<ActionResponse>, MainboardError> {
        return self.send_cmd("pmconfig", &format!("{}/{}", id, prop), config, actor);
    }

    fn send_cmd(
        &self,
        cmd: &'static str,
        id: &str,
        config: Box<dyn ModuleValueParsable>,
        actor: Actor,
    ) -> Result<tokio::sync::oneshot::Receiver<ActionResponse>, MainboardError> {
        let (sender, receiver) = tokio::sync::oneshot::channel::<ActionResponse>();
        let cmd = ModuleStateCmd {
            cmd: cmd.into(),
            topic: format!("local:/{}", id),
            sender,
            actor,
            data: std::sync::Arc::new(config.write_to_bytes()?),
        };
        self.sender_module
            .try_send(ModuleMsg::Cmd(cmd))
            .map_err(|x| MainboardError::from_error(x.to_string()))?;
        return Ok(receiver);
    }
}
