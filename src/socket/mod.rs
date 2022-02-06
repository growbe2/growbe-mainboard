
pub mod mqtt;


use std::sync::mpsc::{Receiver};
use std::sync::{Mutex, Arc};
use std::time::{Instant, Duration};
use rumqtt::{MqttOptions, MqttClient, QoS, Notification};
use protobuf::Message;

struct SocketMessagingError {
    pub status: i32,
}

impl SocketMessagingError {
    pub fn new() -> Self {
        return SocketMessagingError{
            status: 1, // TODO use a define for default error code
        };
    }

    pub fn status(mut self, status: i32) -> SocketMessagingError {
        self.status = status;
        self
    }
}

struct MqttHandler {
    pub subscription: String,
    pub regex: &'static str,
    pub action_code: crate::protos::message::ActionCode,
    pub handler: fn(topic_name: String, data: Arc<Vec<u8>>) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError>,
    pub not_prefix: bool,
}

fn get_topic_prefix(subtopic: & str) -> String {
    return format!("/growbe/{}{}",crate::id::get(), subtopic);
}

fn on_set_rtc(_topic_name: String, data: Arc<Vec<u8>>) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let payload = crate::protos::message::RTCTime::parse_from_bytes(&data).unwrap();
    crate::mainboardstate::rtc::set_rtc(payload);
    Err(SocketMessagingError::new().status(400))
}

/*
fn on_sync_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd{
        cmd: "sync",
        data: data,
        topic: topic_name,
    }).unwrap();
    None
}

fn on_mconfig_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd{
        cmd: "mconfig",
        topic: topic_name,
        data: data,
    }).unwrap();
    None
}

fn on_rmconfig_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd{
        cmd: "rmconfig",
        topic: topic_name,
        data: data,
    }).unwrap();
    None
}

fn on_add_alarm_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "aAl",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_remove_alarm_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "rAl",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_start_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "startCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_set_step_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "setCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_confirm_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "terminateCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_cancel_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "cancelCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_status_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "statusCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_add_vr(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "addVr",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_update_vr(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "updateVr",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}


fn on_rm_vr(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "rmVr",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_config_vr(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>, bool)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "configVr",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}
*/

fn on_update(_topic_name: String, data: Arc<Vec<u8>>) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let payload = crate::protos::board::VersionRelease::parse_from_bytes(&data).unwrap();
    let update_executed_result = crate::mainboardstate::update::handle_version_update(&payload);
    if let Some(update_executed) = update_executed_result {
        return Ok(Some((format!("/growbe/{}/updated", crate::id::get()), update_executed.write_to_bytes().unwrap(), true)));
    }
    return Ok(None);
}

fn on_restart(_topic_name: String, _data: Arc<Vec<u8>>) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let d = tokio::task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        crate::plateform::restart::restart_process();
    });
    return Ok(None);
}

fn on_reboot(_topic_name: String, _data: Arc<Vec<u8>>) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let d = tokio::task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        crate::plateform::restart::restart_host();
    });
    return Ok(None);
}

fn on_helloworld(_topic_name: String, _data: Arc<Vec<u8>>) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let hello_world = crate::mainboardstate::hello_world::get_hello_world();
    return Some((format!("/growbe/{}/hello", crate::id::get()), hello_world.write_to_bytes().unwrap(), true));
}

fn on_localconnection(_topic_name: String, _data: Arc<Vec<u8>>) -> Result<Option<(String, Vec<u8>, bool)>, SocketMessagingError> {
    let local_connection = crate::mainboardstate::localconnection::get_local_connection();
    return Some((format!("/growbe/{}/localconnection", crate::id::get()), local_connection.write_to_bytes().unwrap(), true));
}

pub fn socket_task(
    receiver_socket: Arc<Mutex<Receiver<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>>>,
    config_mqtt: &'static mqtt::CloudMQTTConfig,
) -> tokio::task::JoinHandle<()> {
    /*
    Handle pour les trucs mqtt, ca va s'executer dans une task async et faut je fasse le map
     */
    let handlers = vec!(
        MqttHandler{
            subscription: String::from("/board/setTime"),
            regex: "setTime",
            action_code: crate::protos::message::ActionCode::RTC_SET,
            handler: on_set_rtc,
            not_prefix: false,
        },
        MqttHandler{
            subscription: String::from("/board/helloworld"),
            regex: "helloworld",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_helloworld,
            not_prefix: false,
        },
        MqttHandler{
            subscription: String::from("/board/localconnection"),
            regex: "localconnection",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_localconnection,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/restart".to_string(),
            regex: "restart",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_restart,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/reboot".to_string(),
            regex: "reboot",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_reboot,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/update".to_string(),
            regex: "update",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_update,
            not_prefix: true,
        }
    );

    let mapping_module = vec![
        "sync",
        "mconfig",
        "rmconfig",
        "aAl",
        "rAl",
        "addVr",
        "configVr",
        "rmVr",
        "startCalibration",
        "setCalibration",
        "terminateCalibration",
        "cancelCalibration",
        "statusCalibration"
    ]

    let id_client = format!("pi-{}", crate::id::get());


    let (receiver_action_response, sender_action_response) = std::sync::mpsc::channel<crate::protos::message::ActionResponse>::();

   return tokio::spawn(async move {
        let hearth_beath_rate = Duration::from_secs(15);

        let (mut client, notifications) = loop {
            let config = MqttOptions::new(id_client.clone(), config_mqtt.url.as_str(), config_mqtt.port);
            match MqttClient::start(config) {
                Ok(v) => {
                    break v;
                },
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
                client.subscribe(handler.subscription.as_str(),  QoS::ExactlyOnce).unwrap();
            } else {
                client.subscribe(format!("/growbe/{}{}", crate::id::get(), handler.subscription),  QoS::ExactlyOnce).unwrap();
            }
        });

        mapping_module.iter().for_each(|handler| {
            client.subscribe(format!("/growbe/{}{}", crate::id::get(), handler),  QoS::ExactlyOnce).unwrap();
        });

        loop {
            {
                let receive = receiver_socket.lock().unwrap().try_recv();
                if receive.is_ok() {
                    let message = receive.unwrap();
                    let payload = message.1.write_to_bytes().unwrap();
                    client.publish(get_topic_prefix(message.0.as_str()), QoS::ExactlyOnce, false, payload).unwrap();
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
                                let handler_result = (item.handler)(String::from(d.topic_name.as_str()), d.payload);
                                action_respose.action = item.action_code;
                                if let Ok(result) = handler_result {
                                    if let Some(ret) = result {
                                        client.publish(ret.0, QoS::ExactlyOnce, false, ret.1).unwrap();
                                    }
                                    action_respose.status = 0;
                                    action_respose.msg = String::from("");
                                } else {
                                    //let err = handler_result.error();
                                    action_respose.status = 400;
                                    action_respose.msg = String::from("error");

                                }
                            } else {
                                let module_cmd_result = mapping_module.iter().find(|&x| {
                                    return d.topic_name.contains(x);
                                });
                                if let Some(cmd) = module_cmd_result {
                                    // send to modulestate handler and wait for response
                                    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
                                        cmd: cmd,
                                        topic: d.topic_name,
                                        data: d.payload,
                                        sender: sender_action_response.clone(),
                                    }).unwrap();
                                    None
                                } else {
                                    action_respose.status = 401;
                                }
                            }

                           client.publish(format!("{}{}", d.topic_name.as_str(), "/response"), QoS::ExactlyOnce, false, action_respose.write_to_bytes().unwrap()).unwrap();
                            
                            last_send_instant = Instant::now();
                        },
                        Notification::Reconnection => {
                            log::warn!("mqtt reconnection");
                            handlers.iter().for_each(|handler| client.subscribe(format!("/growbe/{}{}", crate::id::get(), handler.subscription),  QoS::ExactlyOnce).unwrap());
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
                    
                    client.publish(get_topic_prefix("/heartbeath"), QoS::ExactlyOnce, false, payload).unwrap();

                    last_send_instant = Instant::now();

                    log::debug!("sending hearthbeath");
               }
            }
        }    
    });
}