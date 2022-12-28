use std::collections::HashMap;

use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

use crate::{
    mainboardstate::error::MainboardError,
    modulestate::interface::{ModuleStateCmd, ModuleValueParsable, ModuleMsg},
    protos::{message::ActionResponse, module::{RelayOutletConfig, Actor, ModuleActorOwnershipRequest}, env_controller::MActor},
};

pub struct ModuleCommandSender {
    sender_module: Sender<ModuleMsg>,
}

impl crate::modulestate::interface::ModuleValue for ModuleActorOwnershipRequest {}

impl crate::modulestate::interface::ModuleValueParsable for ModuleActorOwnershipRequest {}

impl ModuleCommandSender {
    pub fn new(sender_module: Sender<ModuleMsg>) -> Self {
        Self {
            sender_module,
        }
    }

    pub fn send_relay_config(
        &self,
        _type_relay: &str,
        _id: &str,
        _config: RelayOutletConfig,
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

    pub fn send_reserve_actor(
        &self,
        mactors: Vec<MActor>,
        actor: Actor,
        state: bool,
    ) -> Result<Vec<Receiver<ActionResponse>>, MainboardError> {
        let mut maps: HashMap<String, HashMap<String, bool>> = HashMap::new();

        for actor in mactors {
            if !maps.contains_key(actor.get_id()) {
                maps.insert(actor.get_id().into(), HashMap::new());
            }
            maps.get_mut(actor.get_id()).unwrap().insert(actor.property, state);
        }
        let items: Vec<Receiver<ActionResponse>> = maps.into_iter().map(|(k, v)| {
            let mut request = ModuleActorOwnershipRequest::new();
            request.property = v;
            self.send_cmd("oconfig", k.as_str(), Box::new(request), actor.clone()).unwrap()
        }).collect();
        
        return Ok(items);
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
