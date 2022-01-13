
pub mod mqtt;


use std::sync::mpsc::{Receiver};
use std::sync::{Mutex, Arc};
use std::time::{Instant, Duration};
use rumqtt::{MqttOptions, MqttClient, QoS, Notification};
use protobuf::Message;


struct MqttHandler {
    pub subscription: String,
    pub regex: &'static str,
    pub action_code: crate::protos::message::ActionCode,
    pub handler: fn(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)>,
    pub not_prefix: bool,
}

fn get_topic_prefix(subtopic: & str) -> String {
    return format!("/growbe/{}{}",crate::id::get(), subtopic);
}

fn on_set_rtc(_topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    let payload = crate::protos::message::RTCTime::parse_from_bytes(&data).unwrap();
    crate::mainboardstate::rtc::set_rtc(payload);
    None
}

fn on_sync_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd{
        cmd: "sync",
        data: data,
        topic: topic_name,
    }).unwrap();
    None
}

fn on_mconfig_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd{
        cmd: "mconfig",
        topic: topic_name,
        data: data,
    }).unwrap();
    None
}

fn on_add_alarm_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "aAl",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_remove_alarm_request(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "rAl",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_start_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "startCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_set_step_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "setCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_confirm_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "terminateCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_cancel_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "cancelCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_status_calibration(topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    crate::modulestate::CHANNEL_MODULE_STATE_CMD.0.lock().unwrap().send(crate::modulestate::interface::ModuleStateCmd {
        cmd: "statusCalibration",
        topic: topic_name,
        data: data
    }).unwrap();
    None
}

fn on_update(_topic_name: String, data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    let payload = crate::protos::board::VersionRelease::parse_from_bytes(&data).unwrap();
    let update_executed_result = crate::mainboardstate::update::handle_version_update(&payload);
    if let Some(update_executed) = update_executed_result {
        return Some((format!("/growbe/{}/updated", crate::id::get()), update_executed.write_to_bytes().unwrap()));
    }
    return None;
}

fn on_restart(_topic_name: String, _data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    let d = tokio::task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        crate::plateform::restart::restart_process();
    });
    return None;
}

fn on_reboot(_topic_name: String, _data: Arc<Vec<u8>>) -> Option<(String, Vec<u8>)> {
    let d = tokio::task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        crate::plateform::restart::restart_host();
    });
    return None;
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
            subscription: String::from("/board/sync"),
            regex: "sync",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_sync_request,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/mconfig/+".to_string(),
            regex: "mconfig",
            action_code: crate::protos::message::ActionCode::MODULE_CONFIG,
            handler: on_mconfig_request,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/aAl".to_string(),
            regex: "aAl",
            action_code: crate::protos::message::ActionCode::ADD_ALARM,
            handler: on_add_alarm_request,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/rAl".to_string(),
            regex: "rAl",
            action_code: crate::protos::message::ActionCode::REMOVE_ALARM,
            handler: on_remove_alarm_request,
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
        },
        MqttHandler{
            subscription: "/board/startCalibration/+".to_string(),
            regex: "startCalibration",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_start_calibration,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/setCalibration/+".to_string(),
            regex: "setCalibration",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_set_step_calibration,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/terminateCalibration/+".to_string(),
            regex: "terminateCalibration",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_confirm_calibration,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/cancelCalibration/+".to_string(),
            regex: "cancelCalibration",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_cancel_calibration,
            not_prefix: false,
        },
        MqttHandler{
            subscription: "/board/statusCalibration/+".to_string(),
            regex: "statusCalibration",
            action_code: crate::protos::message::ActionCode::SYNC_REQUEST,
            handler: on_status_calibration,
            not_prefix: false,
        }
    );

    let id_client = format!("pi-{}", crate::id::get());

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
                            let item = handlers.iter().find(|&x| {
                                return d.topic_name.contains(x.regex);
                            }).unwrap();

                            if let Some(ret) = (item.handler)(String::from(d.topic_name.as_str()), d.payload) {
                                client.publish(ret.0, QoS::ExactlyOnce, false, ret.1).unwrap();
                            } else {
                                let mut action_respose = crate::protos::message::ActionResponse::new();
                                action_respose.action = item.action_code;
                                action_respose.status = 0;
                                action_respose.msg = String::from("");
                                client.publish(format!("{}{}", d.topic_name.as_str(), "/response"), QoS::ExactlyOnce, false, action_respose.write_to_bytes().unwrap()).unwrap();
                            }
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