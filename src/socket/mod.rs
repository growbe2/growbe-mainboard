use std::sync::mpsc::{Receiver};
use std::sync::{Mutex, Arc};
use std::time::{Instant, Duration};
use rumqtt::{MqttOptions, MqttClient, QoS, Notification};
use protobuf::Message;

struct MqttHandler {
    pub subscription: String,
    pub regex: &'static str,
    pub action_code: crate::protos::message::ActionCode,
    pub handler: fn(topic_name: String, data: Arc<Vec<u8>>) -> (),
}


fn on_set_rtc(topic_name: String, data: Arc<Vec<u8>>) -> () {
    let payload = crate::protos::message::RTCTime::parse_from_bytes(&data).unwrap();
    crate::mainboardstate::rtc::set_rtc(payload);
}


pub fn socket_task(
    receiver_socket: Arc<Mutex<Receiver<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>>>,
) -> tokio::task::JoinHandle<()> {
    /*
    Handle pour les trucs mqtt, ca va s'executer dans une task async et faut je fasse le map
     */

    let handlers = vec!(
        MqttHandler{
            subscription: String::from("/board/setTime"),
            regex: "setTime",
            action_code: crate::protos::message::ActionCode::RTC_SET,
            handler: on_set_rtc
        }
    );

   return tokio::spawn(async move {
        let hearth_beath_rate = Duration::from_secs(5);
        let mqtt_options = MqttOptions::new("rumqtt-mainboard", "broker.dev.growbe.ca", 1883);
        let (mut client, notifications) = MqttClient::start(mqtt_options).unwrap();

        let mut last_send_instant = Instant::now();

        handlers.iter().for_each(|handler| client.subscribe(format!("/growbe/{}{}", crate::id::get_id(), handler.subscription),  QoS::ExactlyOnce).unwrap());

        let mut send = |topic, payload, includeprefix| -> Instant {
            let full_topic = format!("{}{}{}", if includeprefix { "/growbe/" } else { "" },crate::id::get_id() ,topic);
            client.publish(full_topic, QoS::ExactlyOnce, false, payload).unwrap();
            return Instant::now();
        };

        loop {
            {
                let receive = receiver_socket.lock().unwrap().try_recv();
                if receive.is_ok() {
                    let message = receive.unwrap();
                    let payload = message.1.write_to_bytes().unwrap();
                    last_send_instant = send(message.0, payload, true);
                }
            }
            {
                let incomming_message_result = notifications.try_recv();
                if incomming_message_result.is_ok() {
                    let message = incomming_message_result.unwrap();
                    match message {
                        Notification::Publish(d) => {
                            println!("Receive message from cloud, {}", d.topic_name);
                            let item = handlers.iter().find(|&x| {
                                return d.topic_name.contains(x.regex);
                            }).unwrap();

                            (item.handler)(String::from(d.topic_name.as_str()), d.payload);

                            // send back the reponse
                            let mut action_respose = crate::protos::message::ActionResponse::new();
                            action_respose.action = item.action_code;
                            action_respose.status = 0;
                            action_respose.msg = String::from("");
                            send(format!("{}{}", d.topic_name.as_str(), "/response"), action_respose.write_to_bytes().unwrap(), false);

                        },
                        _ => println!("Oupsy"),
                    }
                    
                }
            }
            {
                if last_send_instant.elapsed() > hearth_beath_rate {
                    let hearth_beath = crate::protos::message::HearthBeath::new();
                    let payload = hearth_beath.write_to_bytes().unwrap();
                    last_send_instant = send(String::from("/hearthbeat"), payload, true);
                    println!("Sending hearthbeath");
               }
            }
        }    
    });
}