use crate::{
    mainboardstate::error::MainboardError,
    modulestate::controller::{
            context::Context, controller_trait::EnvControllerTask,
            module_command::ModuleCommandSender,
    },
    protos::{
        alarm::{AlarmZone, FieldAlarmEvent},
        env_controller::{
            EnvironmentControllerConfiguration_oneof_implementation, EnvironmentControllerEvent,
            EnvironmentControllerState, MActor, MObserver, SCConditionActor,
        },
        message::ActionCode,
        module::{RelayModuleConfig, RelayOutletConfig},
    },
    send_event,
};
use protobuf::{ProtobufEnum, RepeatedField};
use tokio::{select, sync::watch::Receiver};

impl crate::modulestate::interface::ModuleValue for RelayOutletConfig {}
impl crate::modulestate::interface::ModuleValueParsable for RelayOutletConfig {}

pub struct StaticControllerImplementation {}

impl StaticControllerImplementation {
    pub fn new() -> Self {
        Self {}
    }
}

fn get_config_for_event(
    actors: &RepeatedField<MActor>,
    field_value: &FieldAlarmEvent,
    action: &SCConditionActor,
) -> Vec<(String, String, RelayOutletConfig)> {
    let index: i32 = field_value.currentZone.value();
    return action
        .actions
        .clone()
        .into_iter()
        .map(|(k, v)| {
            if let Some(actor) = actors.iter().find(|x| x.name == k) {
                if let Some(item) = v.config.get(&index) {
                    Some((actor.id.clone(), actor.property.clone(), item.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect();
    /*if let Some(item) = action.actions.get(&index) {
        if !item.config.is_empty() {
            let items: Vec<(String, String, RelayOutletConfig)> = item.config.clone().into_iter().map(|(k,v)| {
                let actor = actors.iter().find(|x| x.name == k).unwrap();
                (actor.id.clone(), actor.property.clone(), v)
            }).collect();
            return Some(items);
            //let observer = get_env_element!(ctx, observers, observer_id).unwrap();
            //let v = item.config.clone().unwrap();
            //if v.has_alarm() || v.has_manual() || v.has_cycle() {
            //    return Some(v);
            //}
        }
    }*/
}

fn on_value_event_change(
    context: &ModuleCommandSender,
    receiver_alarm: &mut Receiver<FieldAlarmEvent>,
    action: &SCConditionActor,
    actors: &RepeatedField<MActor>,
) {
    let initial_value = receiver_alarm.borrow_and_update().clone();
    let config_relays = get_config_for_event(&actors, &initial_value, action);
    for (k, p, config_relay) in config_relays {
        context
            .send_mconfig_prop(&k, &p, Box::new(config_relay))
            .unwrap();
    }
}

impl EnvControllerTask for StaticControllerImplementation {
    fn run(
        &self,
        ctx: Context,
    ) -> Result<tokio::task::JoinHandle<Result<(), MainboardError>>, MainboardError> {
        let mut ctx = ctx;
        return Ok(tokio::task::spawn(async move {
            log::info!("starting static controller : {}", ctx.config.get_id());

            let imple = ctx.config.implementation.clone().unwrap();
            let imple = match imple {
                EnvironmentControllerConfiguration_oneof_implementation::field_static(s) => {
                    s.clone()
                }
                _ => panic!("failed to be"),
            };
            //let action = imple.conditions.get(0).unwrap();
            //let observer_id = action.get_observer_id();
            //let actor_id = action.get_actor_id();
            //let observer = get_env_element!(ctx, observers, observer_id).unwrap();
            //let actor = get_env_element!(ctx, actors, actor_id).unwrap();
            //let key = format!("{}:{}", observer.get_id(), observer.get_property());

            //let mut receiver_alarm = ctx.alarm_receivers.get_mut(&key).unwrap();

            //on_value_event_change(
            //    &ctx.module_command_sender,
            //    &mut receiver_alarm,
            //    &action,
            //    &actor,
            //);

            for (_, mut receiver_alarm) in ctx.alarm_receivers.iter_mut() {
                for action in imple.conditions.iter() {
                    on_value_event_change(
                        &ctx.module_command_sender,
                        &mut receiver_alarm,
                        &action,
                        &ctx.config.actors,
                    );
                }
            }

            //send_event!(ctx, EnvironmentControllerState::CHANGING_CONFIG, true);

            send_event!(ctx, EnvironmentControllerState::WAITING_ALARM, true);

            loop {
                for (_k, mut receiver_alarm) in ctx.alarm_receivers.iter_mut() {
                    if let Ok(recv) = receiver_alarm.has_changed() {
                        if recv {
                            println!("has changed");
                            for action in imple.conditions.iter() {
                                on_value_event_change(
                                    &ctx.module_command_sender,
                                    &mut receiver_alarm,
                                    &action,
                                    &ctx.config.actors,
                                );
                                send_event!(ctx, EnvironmentControllerState::CHANGING_CONFIG, true);
                            }
                        }
                    }
                }
                if ctx.cancellation_token.is_cancelled() {
                    log::info!("static controller stopped");
                    send_event!(ctx, EnvironmentControllerState::SLEEPING, false);
                    return Ok(());
                }
                /*
                select! {
                    _ = ctx.cancellation_token.cancelled() => {
                        log::info!("static controller stopped");
                        send_event!(ctx, EnvironmentControllerState::SLEEPING, false);
                        return Ok(());
                    },
                    res = receiver_alarm.changed() => {
                        if res.is_ok() {
                            on_value_event_change(&ctx.module_command_sender, &mut receiver_alarm, &action, &actor);
                            send_event!(ctx, EnvironmentControllerState::CHANGING_CONFIG, true);
                        }
                    },
                    _ = tokio::time::sleep(std::time::Duration::from_millis(5000)) => {
                        println!("still alive");
                    }
                }
                */
            }
        }));
    }
}

//#[cfg(test)]
mod tests {

    use std::{collections::HashMap, hash::Hash, time::Duration};

    use protobuf::RepeatedField;
    use tokio::sync::watch::{channel, Sender};
    use tokio_util::sync::CancellationToken;

    use crate::{
        modulestate::{
            alarm::model::ModuleValueChange,
            controller::context::Context, controller::module_command::ModuleCommandSender, interface::ModuleMsg,
        },
        protos::{
            alarm::{AlarmZone, FieldAlarmEvent},
            env_controller::{
                EnvironmentControllerConfiguration, MObserver, RessourceType, SCConditionActor,
                SCObserverAction,
            },
            module::ManualConfig,
        },
        socket::ss::SenderPayload,
    };

    //use serial_test::serial;

    use super::*;

    fn init(
        module_id: &str,
        property: &str,
        module_actor_id: &str,
        actor_property: &str,
        condition: SCConditionActor,
    ) -> (
        Context,
        Sender<FieldAlarmEvent>,
        Sender<ModuleValueChange<f32>>,
        tokio::sync::mpsc::Receiver<SenderPayload>,
        tokio::sync::mpsc::Receiver<ModuleMsg>,
        EnvironmentControllerConfiguration,
        CancellationToken,
    ) {
        let (sa, ra) = channel(FieldAlarmEvent {
            moduleId: module_id.into(),
            property: property.into(),
            currentZone: AlarmZone::UNKNOW,
            ..Default::default()
        });
        let (sm, rm) = channel(ModuleValueChange::<f32> {
            module_id: module_id.into(),
            changes: vec![],
        });
        let (s_module, r_module) = tokio::sync::mpsc::channel(4);
        let mut alarm_receivers = HashMap::new();
        let mut value_receivers = HashMap::new();

        alarm_receivers.insert("AAA0000003:airTemperature".into(), ra);
        value_receivers.insert("AAA0000003".into(), rm);

        let mut config = EnvironmentControllerConfiguration::new();
        config.set_id("test".to_string());

        config.mut_observers().push(MObserver {
            name: "test_observer".into(),
            id: module_id.into(),
            property: property.to_string(),
            field_type: crate::protos::env_controller::RessourceType::ACTOR_MODULE,
            ..Default::default()
        });

        config.mut_actors().push(MActor {
            name: "test_actor".into(),
            id: module_actor_id.into(),
            property: actor_property.into(),
            field_type: RessourceType::ACTOR_MODULE,
            ..Default::default()
        });

        let mut conditions = RepeatedField::new();
        conditions.push(condition);
        config.set_field_static(
            crate::protos::env_controller::StaticControllerImplementation {
                conditions,
                ..Default::default()
            },
        );

        let cancellation_token = CancellationToken::new();

        let (ss, sr) = tokio::sync::mpsc::channel::<SenderPayload>(5);

        return (
            Context {
                config: config.clone(),
                cancellation_token: cancellation_token.clone(),
                module_command_sender: ModuleCommandSender::new(s_module),
                alarm_receivers,
                value_receivers,
                sender_socket: ss.into(),
            },
            sa,
            sm,
            sr,
            r_module,
            config,
            cancellation_token,
        );
    }

    //#[tokio::test]
    //#[serial]
    async fn env_controller_static_start_and_stop() {
        let mut condition = SCConditionActor::default();
        condition.observer_id = "test_observer".into();
        let (ctx, sa, sm, mut sr, _rm, config, ct) = init(
            "AAA0000003",
            "airTemperature",
            "AAP0000003",
            "p0",
            condition,
        );

        let imple = StaticControllerImplementation::new();

        let handle = imple.run(ctx).unwrap();

        assert_eq!(handle.is_finished(), false);
        assert_eq!(ct.is_cancelled(), false);

        ct.cancel();
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(handle.is_finished(), true);
        assert_eq!(ct.is_cancelled(), true);

        /*
        let d = sr.recv().await.unwrap().1;
        let first_message = d
            .as_any()
            .downcast_ref::<EnvironmentControllerEvent>()
            .unwrap();
        assert_eq!(
            first_message.state,
            EnvironmentControllerState::WAITING_ALARM
        );
        assert_eq!(first_message.running, true);

        let d = sr.recv().await.unwrap().1;
        let first_message = d
            .as_any()
            .downcast_ref::<EnvironmentControllerEvent>()
            .unwrap();
        assert_eq!(first_message.state, EnvironmentControllerState::SLEEPING);
        assert_eq!(first_message.running, false);
        */
    }

    //#[tokio::test]
    //#[serial]
    async fn env_controller_static_reat_alarm_undefined_zone_dont_send() {
        let mut condition = SCConditionActor::default();
        condition.observer_id = "test_observer".into();
        let (ctx, sa, sm, sr, _rm, config, ct) = init(
            "AAA0000003",
            "airTemperature",
            "AAP0000003",
            "p0",
            condition,
        );

        let imple = StaticControllerImplementation::new();

        let handle = imple.run(ctx).unwrap();

        assert_eq!(handle.is_finished(), false);
        assert_eq!(ct.is_cancelled(), false);

        sa.send(FieldAlarmEvent {
            moduleId: "AAA0000003".into(),
            property: "airTemperature".into(),
            ..Default::default()
        })
        .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(handle.is_finished(), false);

        //let result = CHANNEL_MODULE_STATE_CMD
        //    .1
        //    .lock()
        //    .unwrap()
        //    .try_recv();
        //assert_eq!(result.is_err(), true);

        ct.cancel();
    }

    //#[tokio::test]
    //#[serial]
    async fn env_controller_static_reat_alarm_defined_zone_send() {
        let mut condition = SCConditionActor::default();
        condition.observer_id = "test_observer".into();
        let mut actor_action = SCObserverAction::new();
        let mut relay = RelayOutletConfig::new();
        relay.set_manual(ManualConfig {
            state: true,
            ..Default::default()
        });
        let mut map_observer_action = HashMap::new();
        map_observer_action.insert(0, relay);
        actor_action.set_config(map_observer_action);
        condition
            .actions
            .insert("test_actor".into(), actor_action);

        let (ctx, sa, sm, sr, _rm, config, ct) = init(
            "AAA0000003",
            "airTemperature",
            "AAP0000003",
            "p0",
            condition,
        );

        let imple = StaticControllerImplementation::new();

        let handle = imple.run(ctx).unwrap();

        assert_eq!(handle.is_finished(), false);
        assert_eq!(ct.is_cancelled(), false);

        sa.send(FieldAlarmEvent {
            moduleId: "AAA0000003".into(),
            property: "airTemperature".into(),
            ..Default::default()
        })
        .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(handle.is_finished(), false);


        // TODO :fix
        /*let cmd = CHANNEL_MODULE_STATE_CMD
            .1
            .lock()
            .unwrap()
            .try_recv()
            .unwrap();

        assert_eq!(cmd.cmd, "pmconfig");
        */

        ct.cancel();
    }
}
