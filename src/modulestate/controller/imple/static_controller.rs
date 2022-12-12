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
            EnvironmentControllerState, MActor, SCConditionActor,
        },
        module::RelayOutletConfig, message::ActionCode,
    },
    send_event,
};
use protobuf::ProtobufEnum;
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
    field_value: &FieldAlarmEvent,
    action: &SCConditionActor,
) -> Option<RelayOutletConfig> {
    let index: i32 = field_value.currentZone.value();
    if let Some(item) = action.actions.get(&index) {
        if item.config.is_some() {
            let v = item.config.clone().unwrap();
            if v.has_alarm() || v.has_manual() || v.has_cycle() {
                return Some(v);
            }
        }
    }
    return None;
}

fn on_value_event_change(
    code: AlarmZone,
    context: &ModuleCommandSender,
    receiver_alarm: &mut Receiver<FieldAlarmEvent>,
    action: &SCConditionActor,
    actor: &MActor,
) -> AlarmZone {
    let initial_value = receiver_alarm.borrow();
    if initial_value.currentZone != AlarmZone::UNKNOW && initial_value.currentZone == code {
        return code;
    }
    if let Some(config_relay) = get_config_for_event(&initial_value, action) {
        context
            .send_mconfig_prop(&actor.id, &actor.property, Box::new(config_relay))
            .unwrap();
    } else {
        log::info!(
            "no configuration for new alarm zone {:?}",
            initial_value.currentZone
        );
    }

    return initial_value.currentZone;
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
            let action = imple.conditions.get(0).unwrap();

            let observer_id = action.get_observer_id();
            let actor_id = action.get_actor_id();
            let observer = get_env_element!(ctx, observers, observer_id).unwrap();
            let actor = get_env_element!(ctx, actors, actor_id).unwrap();
            let key = format!("{}:{}", observer.get_id(), observer.get_property());

            let mut receiver_alarm = ctx.alarm_receivers.remove(&key).unwrap();


            let mut last_zone = on_value_event_change(
                AlarmZone::UNKNOW,
                &ctx.module_command_sender,
                &mut receiver_alarm,
                &action,
                &actor,
            );

            send_event!(ctx, EnvironmentControllerState::CHANGING_CONFIG, true);

            loop {
                send_event!(ctx, EnvironmentControllerState::WAITING_ALARM, true);

                select! {
                    _ = ctx.cancellation_token.cancelled() => {
                        log::info!("static controller stopped");
                        send_event!(ctx, EnvironmentControllerState::SLEEPING, false);
                        return Ok(());
                    },
                    _ = receiver_alarm.changed() => {
                        let new_zone = on_value_event_change(last_zone,&ctx.module_command_sender, &mut receiver_alarm, &action, &actor);
                        if last_zone != new_zone {
                            log::info!("receive alarm changed");
                            send_event!(ctx, EnvironmentControllerState::CHANGING_CONFIG, true);
                            last_zone = new_zone;
                        }
                    }
                }
            }
        }));
    }
}

#[cfg(test)]
mod tests {

    use std::{collections::HashMap, time::Duration};

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
        condition.observer_id = "test_observer".into();
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
            EnvironmentControllerState::CHANGING_CONFIG
        );
        assert_eq!(first_message.running, true);

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
        condition.observer_id = "test_observer".into();
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
        condition.observer_id = "test_observer".into();
        condition.actor_id = "test_actor".into();
        let mut actor_action = SCObserverAction::new();
        let mut relay = RelayOutletConfig::new();
        relay.set_manual(ManualConfig {
            state: true,
            ..Default::default()
        });
        actor_action.set_config(relay);
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
