pub mod http;
pub mod mqtt;
pub mod ss;

use protobuf::Message;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, SubscribeFilter};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Instant;

use crate::comboard::imple::virt::VirtualScenarioItem;
use crate::mainboardstate::config::rewrite_configuration;
use crate::mainboardstate::error::MainboardError;
use crate::modulestate::interface::ModuleMsg;
use crate::protos::message::ActionCode;
use crate::protos::module::{Actor, ActorType};

struct SocketMessagingError {
    pub status: u32,
    pub msg: String,
}

impl SocketMessagingError {
    pub fn new() -> Self {
        return SocketMessagingError {
            status: 1, // TODO use a define for default error code
            msg: "".to_string(),
        };
    }

    pub fn status(mut self, status: u32) -> SocketMessagingError {
        self.status = status;
        self
    }

    pub fn message(mut self, msg: String) -> SocketMessagingError {
        self.msg = msg;
        self
    }
}

impl From<TrySendError<SocketMessagingError>> for SocketMessagingError {
    fn from(value: TrySendError<SocketMessagingError>) -> Self {
        Self {
            status: 1,
            msg: value.to_string(),
        }
    }
}

struct MqttHandler {
    pub subscription: String,
    pub regex: &'static str,
    pub action_code: crate::protos::message::ActionCode,
    pub handler: fn(
        topic_name: String,
        data: Arc<Vec<u8>>,
        ctx: &TaskContext,
    ) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError>,
    pub not_prefix: bool,
}

struct MqttModuleHanlder {
    pub name: String,
    pub suffix: bool,
    pub action_code: ActionCode,
}

struct TaskContext {
    sender_virt: Sender<Vec<VirtualScenarioItem>>,
    sender_module: Sender<ModuleMsg>,
}

impl From<(&str, bool, ActionCode)> for MqttModuleHanlder {
    fn from(value: (&str, bool, ActionCode)) -> Self {
        Self {
            name: value.0.into(),
            suffix: value.1,
            action_code: value.2,
        }
    }
}
lazy_static::lazy_static! {
    static ref MQTT_HANDLES: Vec<MqttHandler> = vec![
        MqttHandler {
            subscription: String::from("/board/setTime"),
            regex: "setTime",
            action_code: crate::protos::message::ActionCode::RTC_SET,
            handler: on_set_rtc,
            not_prefix: false,
        },
        MqttHandler {
            subscription: String::from("/board/helloworld"),
            regex: "helloworld",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_helloworld,
            not_prefix: false,
        },
        MqttHandler {
            subscription: String::from("/board/localconnection"),
            regex: "localconnection",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_localconnection,
            not_prefix: false,
        },
        MqttHandler {
            subscription: "/board/restart".to_string(),
            regex: "restart",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_restart,
            not_prefix: false,
        },
        MqttHandler {
            subscription: "/board/reboot".to_string(),
            regex: "reboot",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_reboot,
            not_prefix: false,
        },
        MqttHandler {
            subscription: "/board/boardconfig".to_string(),
            regex: "boardconfig",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_setconfig,
            not_prefix: false,
        },
        MqttHandler {
            subscription: "/board/virt/item".to_string(),
            regex: "virt",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_virt_item,
            not_prefix: false,
        },
        MqttHandler {
            subscription: "/update".to_string(),
            regex: "update",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_update,
            not_prefix: true,
        },
        MqttHandler {
            subscription: "/update/request".to_string(),
            regex: "update/request",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_update_request,
            not_prefix: true,
        },
    ];

    static ref MAPPING_MODULES: Vec<MqttModuleHanlder> = vec![
        ("sync", false, ActionCode::SYNC_REQUEST).into(),
        ("mconfig", true, ActionCode::MODULE_CONFIG),
        ("rmconfig", true, ActionCode::MODULE_CONFIG),
        ("aEnv", false, ActionCode::SYNC_REQUEST),
        ("rEnv", true, ActionCode::SYNC_REQUEST),
        ("aAl", false, ActionCode::ADD_ALARM),
        ("rAl", false, ActionCode::REMOVE_ALARM),
        ("uAl", false, ActionCode::SYNC_REQUEST),
        ("addVr", false, ActionCode::SYNC_REQUEST),
        ("vrconfig", true, ActionCode::SYNC_REQUEST),
        ("rmVr", true, ActionCode::SYNC_REQUEST),
        ("startCalibration", true, ActionCode::SYNC_REQUEST),
        ("setCalibration", true, ActionCode::SYNC_REQUEST),
        ("terminateCalibration", true, ActionCode::SYNC_REQUEST),
        ("cancelCalibration", true, ActionCode::SYNC_REQUEST),
        ("statusCalibration", true, ActionCode::SYNC_REQUEST),
    ]
    .iter()
    .map(|x| (*x).into())
    .collect();
}

fn get_topic_prefix(subtopic: &str) -> String {
    return format!("/growbe/{}{}", growbe_shared::id::get(), subtopic);
}

fn on_set_rtc(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
    _ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    //let payload = crate::protos::message::RTCTime::parse_from_bytes(&data).unwrap();
    //crate::mainboardstate::rtc::set_rtc(payload);
    Err(SocketMessagingError::new()
        .status(400)
        .message("operation not supported on device".to_string()))
}

fn on_update(
    _topic_name: String,
    data: Arc<Vec<u8>>,
    _ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let payload = crate::protos::board::VersionRelease::parse_from_bytes(&data).unwrap();
    let update_executed_result = crate::mainboardstate::update::handle_version_update(&payload);
    if let Some(update_executed) = update_executed_result {
        return Ok(Some((
            format!("/growbe/{}/updated", growbe_shared::id::get()),
            update_executed.write_to_bytes().unwrap(),
            true,
        )));
    }
    return Ok(None);
}

fn on_update_request(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
    _ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let update_executed_result = crate::mainboardstate::update::handle_version_update_request();
    if let Some(update_executed) = update_executed_result {
        return Ok(Some((
            format!("/growbe/{}/updated", growbe_shared::id::get()),
            update_executed.write_to_bytes().unwrap(),
            true,
        )));
    }
    return Ok(None);
}

fn restart_task() {
    tokio::task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        if let Err(err) = crate::plateform::restart::restart_process() {
            log::error!("failed to restart_process {:?}", err);
        }
    });
}

fn on_restart(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
    _ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    restart_task();
    restart_task();
    return Ok(None);
}

fn reboot_task() {
    tokio::task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        if let Err(err) = crate::plateform::restart::restart_host() {
            log::error!("failed to restart_process {:?}", err);
        }
    });
}

fn on_reboot(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
    _ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    reboot_task();
    reboot_task();
    return Ok(None);
}

fn on_helloworld(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
    _ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let hello_world = crate::mainboardstate::hello_world::get_hello_world();
    return Ok(Some((
        format!("/growbe/{}/hello", growbe_shared::id::get()),
        hello_world.write_to_bytes().unwrap(),
        true,
    )));
}

fn on_localconnection(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
    _ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let local_connection = crate::mainboardstate::localconnection::get_local_connection();
    return Ok(Some((
        format!("/growbe/{}/localconnection", growbe_shared::id::get()),
        local_connection.write_to_bytes().unwrap(),
        true,
    )));
}

fn on_setconfig(
    _topic_name: String,
    data: Arc<Vec<u8>>,
    _ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    match crate::protos::board::MainboardConfig::parse_from_bytes(&data) {
        Ok(config) => {
            rewrite_configuration(config);
            return Ok(None);
        }
        Err(_) => {
            return Err(SocketMessagingError::new());
        }
    }
}

fn on_virt_item(
    _topic_name: String,
    data: Arc<Vec<u8>>,
    ctx: &TaskContext,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    println!("on virt item");

    match serde_json::from_slice(&data) {
        Ok(config) => {
            ctx.sender_virt
                .try_send(config)
                .map_err(|x| SocketMessagingError::new().message(x.to_string()))?;
        }
        Err(_) => {
            return Err(SocketMessagingError::new());
        }
    }

    return Ok(None);
}

async fn handle_subscription_topics(
    client: &AsyncClient,
    handlers: &Vec<MqttHandler>,
    mapping_module: &Vec<MqttModuleHanlder>,
) -> Result<(), MainboardError> {
    for handler in handlers.iter() {
        if handler.not_prefix {
            client
                .subscribe(handler.subscription.as_str(), QoS::ExactlyOnce)
                .await
                .unwrap();
        } else {
            client
                .subscribe(
                    format!(
                        "/growbe/{}{}",
                        growbe_shared::id::get(),
                        handler.subscription
                    ),
                    QoS::ExactlyOnce,
                )
                .await
                .unwrap();
        }
    }

    let topics: Vec<SubscribeFilter> = mapping_module
        .iter()
        .map(|handler| {
            let suffix = if handler.suffix == true { "/+" } else { "" };
            let topic = format!(
                "/growbe/{}/board/{}{}",
                growbe_shared::id::get(),
                handler.name,
                suffix
            );
            SubscribeFilter {
                path: topic,
                qos: QoS::ExactlyOnce,
            }
        })
        .collect();
    client.subscribe_many(topics).await.unwrap();

    Ok(())
}

async fn handle_hearthbeath(
    client: &AsyncClient,
    last_send_instant: Instant,
    duration: Duration,
) -> Instant {
    if last_send_instant.elapsed() >= duration {
        let mut hearth_beath = crate::protos::message::HearthBeath::new();
        let now = chrono::Utc::now();
        hearth_beath.set_rtc(String::from(now.timestamp_nanos().to_string()));
        let payload = hearth_beath.write_to_bytes().unwrap();

        client
            .publish(
                get_topic_prefix("/heartbeath"),
                QoS::ExactlyOnce,
                false,
                payload,
            )
            .await
            .unwrap();

        Instant::now()
    } else {
        last_send_instant
    }
}

async fn handle_incomming_message(
    client: &AsyncClient,
    handlers: &Vec<MqttHandler>,
    mapping_module: &Vec<MqttModuleHanlder>,

    ctx: &TaskContext,

    topic_name: String,
    payload: Arc<Vec<u8>>,
) {
    log::debug!("receive message from cloud, {}", topic_name);
    let item_opt = handlers.iter().find(|&x| {
        return topic_name.contains(x.regex);
    });
    let mut action_respose = crate::protos::message::ActionResponse::new();

    if let Some(item) = item_opt {
        let handler_result = (item.handler)(String::from(topic_name.as_str()), payload, ctx);
        action_respose.action = item.action_code;
        if let Ok(result) = handler_result {
            if let Some(ret) = result {
                client
                    .publish(ret.0, QoS::ExactlyOnce, false, ret.1)
                    .await
                    .unwrap();
            }
            action_respose.status = 0;
            action_respose.set_action(item.action_code);
            action_respose.msg = String::from("");
        } else {
            let err = handler_result.unwrap_err();
            action_respose.status = err.status;
            action_respose.msg = err.msg;
        }
    } else {
        let module_cmd_result = mapping_module.iter().find(|&x| {
            return topic_name.contains(x.name.as_str());
        });

        if let Some(handler) = module_cmd_result {
            let (sender, receiver) = tokio::sync::oneshot::channel();
            let mut actor = Actor::new();
            actor.id = "user".to_string();
            actor.field_type = ActorType::MANUAL_USER_ACTOR;
            ctx.sender_module
                .send(ModuleMsg::Cmd(
                    crate::modulestate::interface::ModuleStateCmd {
                        cmd: handler.name.clone(),
                        topic: topic_name.clone(),
                        data: payload,
                        sender,
                        actor
                    },
                ))
                .await
                .unwrap();

            // wait f
            // or not very long for a response from the state_cmd
            let action_code = handler.action_code;

            println!("waiting for response from module cmd");
            let action_response_result = select! {
                Ok(value) = receiver => {
                    Ok(value)
                },
                _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {
                    Err(())
                }
            };
            match action_response_result {
                Ok(ar) => {
                    action_respose = ar;
                    action_respose.action = action_code;
                }
                Err(_) => {
                    action_respose.action = action_code;
                    action_respose.status = 405;
                    action_respose.msg = "timeout waiting for cmd response".to_string();
                }
            }
        } else {
            action_respose.status = 401;
        }
    }

    if action_respose.action == ActionCode::PARSING {
        action_respose.action = ActionCode::SYNC_REQUEST;
    }
    client
        .publish(
            format!("{}{}", topic_name.as_str(), "/response"),
            QoS::ExactlyOnce,
            false,
            action_respose.write_to_bytes().unwrap(),
        )
        .await
        .unwrap();
    //last_send_instant = Instant::now();
}

pub fn socket_task(
    mut receiver_socket: Receiver<(
        String,
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    )>,
    sender_virt: Sender<Vec<VirtualScenarioItem>>,
    sender_module: Sender<ModuleMsg>,
    config_mqtt: &'static mqtt::CloudMQTTConfig,
) -> tokio::task::JoinHandle<()> {
    let id_client = format!("pi-{}", growbe_shared::id::get());

    let ctx = TaskContext {
        sender_virt,
        sender_module,
    };

    return tokio::spawn(async move {
        let hearth_beath_rate = Duration::from_secs(5);

        let mut config = MqttOptions::new(
            id_client.clone(),
            config_mqtt.url.as_str(),
            config_mqtt.port,
        );
        config.set_keep_alive(Duration::from_secs(5));
        let (client, mut eventloop) = AsyncClient::new(config, 20);

        handle_subscription_topics(&client, &MQTT_HANDLES, &MAPPING_MODULES)
            .await
            .unwrap();

        let mut last_message_at = Instant::now();

        println!("starting listening socket");
        loop {
            select! {
                receive = receiver_socket.recv() => {
                    if receive.is_some() {
                        let message = receive.unwrap();
                        let payload = message.1.write_to_bytes().unwrap();

                        println!("OUTGOING {}", message.0);
                        client
                            .publish(
                                get_topic_prefix(message.0.as_str()),
                                QoS::ExactlyOnce,
                                false,
                            payload,
                        )
                        .await
                        .unwrap();

                        last_message_at = Instant::now();
                    }
                },
                result = eventloop.poll() => {
                    if let Ok(result) = result {
                        match result {
                            Event::Incoming(d) => {
                                match d {
                                    Packet::Publish(message) => {
                                        let data = Arc::new(message.payload.to_vec());
                                        println!("INCOMMING {}", message.topic);
                                        handle_incomming_message(&client, &MQTT_HANDLES, &MAPPING_MODULES, &ctx,  message.topic, data).await;
                                    },
                                    Packet::Connect(c) => {
                                        println!("im connected !!! {:?}", c);
                                    },
                                    _ => {}
                                }
                            },
                            Event::Outgoing(_d) => {
                            }
                        }
                        last_message_at = handle_hearthbeath(&client, last_message_at, hearth_beath_rate).await;
                    } else {
                        println!("errrorororo {:?}", result.unwrap_err());
                    }
                },
                _ = tokio::time::sleep(hearth_beath_rate) => {
                    last_message_at = handle_hearthbeath(&client, last_message_at, hearth_beath_rate).await;
                }
            }
        }
    });
}
