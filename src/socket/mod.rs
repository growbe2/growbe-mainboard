use std::sync::mpsc::{Receiver};
use std::sync::{Mutex, Arc};
use std::time::{Instant, Duration};
use rumqtt::{MqttOptions, MqttClient, QoS};
use protobuf::Message;

pub fn socket_task(
    receiver_socket: Arc<Mutex<Receiver<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>>>,
) -> tokio::task::JoinHandle<()> {

   return tokio::spawn(async move {
        let hearth_beath_rate = Duration::from_secs(5);
        let mqtt_options = MqttOptions::new("rumqtt-mainboard", "broker.dev.growbe.ca", 1883);
        let (mut client, notifications) = MqttClient::start(mqtt_options).unwrap();

        let mut last_send_instant = Instant::now();

        let mut send = |topic, payload| -> Instant {
            let full_topic = format!("/growbe/{}{}", crate::id::get_id() ,topic);
            client.publish(full_topic, QoS::ExactlyOnce, false, payload).unwrap();
            return Instant::now();
        };

        loop {
            {
                let receive = receiver_socket.lock().unwrap().try_recv();
                if receive.is_ok() {
                    let message = receive.unwrap();
                    let payload = message.1.write_to_bytes().unwrap();
                    last_send_instant = send(message.0, payload);
                }
            }
            {
                let incomming_message_result = notifications.try_recv();
                if incomming_message_result.is_ok() {
                    let _message = incomming_message_result.unwrap();
                    println!("Receive message from cloud");
                }
            }
            {
                if last_send_instant.elapsed() > hearth_beath_rate {
                    let hearth_beath = crate::protos::message::HearthBeath::new();
                    let payload = hearth_beath.write_to_bytes().unwrap();
                    last_send_instant = send(String::from("/hearthbeat"), payload);
                    println!("Sending hearthbeath");
               }
            }
        }    
    });
}