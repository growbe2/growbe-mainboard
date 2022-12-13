use crate::{
    get_env_element,
    mainboardstate::error::MainboardError,
    modulestate::controller::{
        context::Context, controller_trait::EnvControllerTask, module_command::ModuleCommandSender,
    },
    protos::{
        alarm::{FieldAlarmEvent, AlarmZone},
        env_controller::{
            EnvironmentControllerConfiguration_oneof_implementation, EnvironmentControllerEvent,
            EnvironmentControllerState, MActor, SCConditionActor, MObserver,
        },
        module::{RelayOutletConfig, RelayModuleConfig}, message::ActionCode,
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
    observers: &RepeatedField<MObserver>,
    field_value: &FieldAlarmEvent,
    action: &SCConditionActor,
) -> Option<Vec<(String, String, RelayOutletConfig)>> {
    let index: i32 = field_value.currentZone.value();
    if let Some(item) = action.actions.get(&index) {
        if !item.config.is_empty() {
            let items: Vec<(String, String, RelayOutletConfig)> = item.config.clone().into_iter().map(|(k,v)| {
                let observer = observers.iter().find(|x| x.name == k).unwrap();
                (observer.id.clone(), observer.property.clone(), v)
            }).collect();
            return Some(items);
            //let observer = get_env_element!(ctx, observers, observer_id).unwrap();
            //let v = item.config.clone().unwrap();
            //if v.has_alarm() || v.has_manual() || v.has_cycle() {
            //    return Some(v);
            //}
        }
    }
    return None;
}

fn on_value_event_change(
    context: &ModuleCommandSender,
    receiver_alarm: &mut Receiver<FieldAlarmEvent>,
    action: &SCConditionActor,
    observers: &RepeatedField<MObserver>,
) {
    let initial_value = receiver_alarm.borrow_and_update().clone();
    if let Some(config_relays) = get_config_for_event(&observers, &initial_value, action) {
        for (k, p, config_relay) in config_relays {
            context
                .send_mconfig_prop(&k, &p, Box::new(config_relay))
                .unwrap();
        }
    } else {
        log::info!(
            "no configuration for new alarm zone {:?}",
            initial_value.currentZone
        );
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


            //send_event!(ctx, EnvironmentControllerState::CHANGING_CONFIG, true);

            send_event!(ctx, EnvironmentControllerState::WAITING_ALARM, true);

            loop {
                for (_k, mut receiver_alarm) in ctx.alarm_receivers.iter_mut() {
                    if let Ok(recv) = receiver_alarm.has_changed() {
                        println!("receive alarm {:?}", recv);
                        if recv {
                            for action in imple.conditions.iter() {
                                on_value_event_change(&ctx.module_command_sender, &mut receiver_alarm, &action, &ctx.config.observers);
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
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
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

#[cfg(test)]
mod tests {

    use std::{collections::HashMap, time::Duration, hash::Hash};

    use protobuf::RepeatedField;
    use tokio::sync::watch::{channel, Sender};
    use tokio_util::sync::CancellationToken;

    use crate::{
        modulestate::{
            alarm::model::ModuleValueChange, cmd::CHANNEL_MODULE_STATE_CMD,
            controller::context::Context, controller::module_command::ModuleCommandSender,
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

    use serial_test::serial;

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
        std::sync::mpsc::Receiver<SenderPayload>,
        EnvironmentControllerConfiguration,
        CancellationToken,
    ) {
        //CHANNEL_MODULE_STATE_CMD.0.clear_poison();
        //CHANNEL_MODULE_STATE_CMD.1.clear_poison();
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

        let (ss, sr) = std::sync::mpsc::channel::<SenderPayload>();

        return (
            Context {
                config: config.clone(),
                cancellation_token: cancellation_token.clone(),
                module_command_sender: ModuleCommandSender::new(),
                alarm_receivers,
                value_receivers,
                sender_socket: ss.into(),
            },
            sa,
            sm,
            sr,
            config,
            cancellation_token,
        );
    }

    #[tokio::test]
    #[serial]
    async fn env_controller_static_start_and_stop() {
        let mut condition = SCConditionActor::default();
        condition.actor_id = "test_actor".into();
        let (ctx, sa, sm, sr, config, ct) = init(
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

        let d = sr.recv_timeout(Duration::from_millis(10)).unwrap().1;
        let first_message = d
            .as_any()
            .downcast_ref::<EnvironmentControllerEvent>()
            .unwrap();
        assert_eq!(
            first_message.state,
            EnvironmentControllerState::WAITING_ALARM
        );
        assert_eq!(first_message.running, true);

        let d = sr.recv_timeout(Duration::from_millis(10)).unwrap().1;
        let first_message = d
            .as_any()
            .downcast_ref::<EnvironmentControllerEvent>()
            .unwrap();
        assert_eq!(first_message.state, EnvironmentControllerState::SLEEPING);
        assert_eq!(first_message.running, false);
    }

    #[tokio::test]
    #[serial]
    async fn env_controller_static_reat_alarm_undefined_zone_dont_send() {
        let mut condition = SCConditionActor::default();
        condition.actor_id = "test_actor".into();
        let (ctx, sa, sm, sr, config, ct) = init(
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

    #[tokio::test]
    #[serial]
    async fn env_controller_static_reat_alarm_defined_zone_send() {
        let mut condition = SCConditionActor::default();
        condition.actor_id = "test_actor".into();
        let mut actor_action = SCObserverAction::new();
        let mut relay = RelayOutletConfig::new();
        relay.set_manual(ManualConfig {
            state: true,
            ..Default::default()
        });
        let mut map_observer_action = HashMap::new();
        map_observer_action.insert("test_observer".into(), relay);
        actor_action.set_config(map_observer_action);
        condition
            .actions
            .insert(AlarmZone::UNKNOW.value(), actor_action);
        let (ctx, sa, sm, sr, config, ct) = init(
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

        let cmd = CHANNEL_MODULE_STATE_CMD
            .1
            .lock()
            .unwrap()
            .try_recv()
            .unwrap();

        assert_eq!(cmd.cmd, "pmconfig");

        ct.cancel();
    }
}
