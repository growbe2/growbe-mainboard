pub mod http;
pub mod mqtt;

use protobuf::Message;
use rumqtt::{MqttClient, MqttOptions, Notification, QoS};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::mainboardstate::config::rewrite_configuration;
use crate::protos::message::ActionCode;

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

struct MqttHandler {
    pub subscription: String,
    pub regex: &'static str,
    pub action_code: crate::protos::message::ActionCode,
    pub handler: fn(
        topic_name: String,
        data: Arc<Vec<u8>>,
    ) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError>,
    pub not_prefix: bool,
}

fn get_topic_prefix(subtopic: &str) -> String {
    return format!("/growbe/{}{}", growbe_shared::id::get(), subtopic);
}

fn on_set_rtc(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
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
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    restart_task();
    restart_task();
    return Ok(None);
}

fn reboot_task() {
    tokio::task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        if let Err(err) = crate::plateform::restart::restart_host() {
            log::error!("failed to restart_process {:?}", err);
        }
    });
}

fn on_reboot(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    reboot_task();
    reboot_task();
    return Ok(None);
}

fn on_helloworld(
    _topic_name: String,
    _data: Arc<Vec<u8>>,
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

pub fn socket_task(
    receiver_socket: Arc<
        Mutex<
            Receiver<(
                String,
                Box<dyn crate::modulestate::interface::ModuleValueParsable>,
            )>,
        >,
    >,
    config_mqtt: &'static mqtt::CloudMQTTConfig,
) -> tokio::task::JoinHandle<()> {
    /*
    Handle pour les trucs mqtt, ca va s'executer dans une task async et faut je fasse le map
     */
    let handlers = vec![
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

    let mapping_module = vec![
        ("sync", false, ActionCode::SYNC_REQUEST),
        ("mconfig", true, ActionCode::MODULE_CONFIG),
        ("rmconfig", true, ActionCode::MODULE_CONFIG),
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
    ];

    let id_client = format!("pi-{}", growbe_shared::id::get());

    let (sender_action_response, receiver_action_response) =
        std::sync::mpsc::channel::<crate::protos::message::ActionResponse>();

    return tokio::spawn(async move {
        let hearth_beath_rate = Duration::from_secs(5);

        let (mut client, notifications) = loop {
            let config = MqttOptions::new(
                id_client.clone(),
                config_mqtt.url.as_str(),
                config_mqtt.port,
            );
            match MqttClient::start(config) {
                Ok(v) => {
                    break v;
                }
                Err(err) => {
                    log::error!("fatal error creating link to the cloud {:?}", err);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            }
        };

        let mut last_send_instant = Instant::now();

        handlers.iter().for_each(|handler| {
            if handler.not_prefix {
                client
                    .subscribe(handler.subscription.as_str(), QoS::ExactlyOnce)
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
                    .unwrap();
            }
        });

        mapping_module.iter().for_each(|handler| {
            let suffix = if handler.1 == true { "/+" } else { "" };
            let topic = format!(
                "/growbe/{}/board/{}{}",
                growbe_shared::id::get(),
                handler.0,
                suffix
            );
            client.subscribe(topic, QoS::ExactlyOnce).unwrap();
        });

        loop {
            {
                let receive = receiver_socket.lock().unwrap().try_recv();
                if receive.is_ok() {
                    let message = receive.unwrap();
                    let payload = message.1.write_to_bytes().unwrap();
                    client
                        .publish(
                            get_topic_prefix(message.0.as_str()),
                            QoS::ExactlyOnce,
                            false,
                            payload,
                        )
                        .unwrap();
                    last_send_instant = Instant::now();
                }
            }
            {
                let incomming_message_result = notifications.try_recv();
                if incomming_message_result.is_ok() {
                    let message = incomming_message_result.unwrap();
                    match message {
                        Notification::Publish(d) => {
                            log::debug!("receive message from cloud, {}", d.topic_name);
                            let item_opt = handlers.iter().find(|&x| {
                                return d.topic_name.contains(x.regex);
                            });

                            let mut action_respose = crate::protos::message::ActionResponse::new();

                            if let Some(item) = item_opt {
                                let handler_result =
                                    (item.handler)(String::from(d.topic_name.as_str()), d.payload);
                                action_respose.action = item.action_code;
                                if let Ok(result) = handler_result {
                                    if let Some(ret) = result {
                                        client
                                            .publish(ret.0, QoS::ExactlyOnce, false, ret.1)
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
                                    return d.topic_name.contains(x.0);
                                });
                                if let Some((cmd, _, action_code)) = module_cmd_result {
                                    // send to modulestate handler and wait for response
                                    crate::modulestate::cmd::CHANNEL_MODULE_STATE_CMD
                                        .0
                                        .lock()
                                        .unwrap()
                                        .send(crate::modulestate::interface::ModuleStateCmd {
                                            cmd: cmd,
                                            topic: d.topic_name.clone(),
                                            data: d.payload,
                                            sender: sender_action_response.clone(),
                                        })
                                        .unwrap();

                                    // wait for not very long for a response from the state_cmd
                                    let action_response_result = receiver_action_response
                                        .recv_timeout(Duration::from_millis(200));
                                    match action_response_result {
                                        Ok(ar) => {
                                            action_respose = ar;
                                            action_respose.action = *action_code;
                                        }
                                        Err(_) => {
                                            action_respose.action = *action_code;
                                            action_respose.status = 405;
                                            action_respose.msg =
                                                "timeout waiting for cmd response".to_string();
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
                                    format!("{}{}", d.topic_name.as_str(), "/response"),
                                    QoS::ExactlyOnce,
                                    false,
                                    action_respose.write_to_bytes().unwrap(),
                                )
                                .unwrap();

                            last_send_instant = Instant::now();
                        }
                        Notification::Reconnection => {
                            log::warn!("mqtt reconnection");
                            handlers.iter().for_each(|handler| {
                                client
                                    .subscribe(
                                        format!(
                                            "/growbe/{}{}",
                                            growbe_shared::id::get(),
                                            handler.subscription
                                        ),
                                        QoS::ExactlyOnce,
                                    )
                                    .unwrap()
                            });
                            handlers.iter().for_each(|handler| {
                                if handler.not_prefix {
                                    client
                                        .subscribe(handler.subscription.as_str(), QoS::ExactlyOnce)
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
                                        .unwrap();
                                }
                            });

                            mapping_module.iter().for_each(|handler| {
                                let suffix = if handler.1 == true { "/+" } else { "" };
                                let topic = format!(
                                    "/growbe/{}/board/{}{}",
                                    growbe_shared::id::get(),
                                    handler.0,
                                    suffix
                                );

                                client.subscribe(topic, QoS::ExactlyOnce).unwrap();
                            });
                        }
                        _ => log::error!("mqtt message not publish {:?}", message),
                    }
                }
            }
            {
                if last_send_instant.elapsed() > hearth_beath_rate {
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
                        .unwrap();

                    last_send_instant = Instant::now();
                }
            }
        }
    });
}
