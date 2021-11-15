use std::sync::mpsc::{Receiver};
use std::sync::{Mutex, Arc};
use rumqtt::{MqttOptions, MqttClient, QoS};


pub fn socket_task(
    receiver_socket: Arc<Mutex<Receiver<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>>>,
) -> tokio::task::JoinHandle<()> {

   return tokio::spawn(async move {
        let mqtt_options = MqttOptions::new("rumqtt-mainboard", "broker.dev.growbe.ca", 1883);
        let (mut client, notifications) = MqttClient::start(mqtt_options).unwrap();

        loop {
            {
                let receive = receiver_socket.lock().unwrap().try_recv();
                if receive.is_ok() {
                    let message = receive.unwrap();
                    let payload = message.1.write_to_bytes().unwrap();
                    client.publish(message.0, QoS::ExactlyOnce, false, payload).unwrap();
                }
            }
            {
                let incomming_message_result = notifications.try_recv();
                if incomming_message_result.is_ok() {
                    let _message = incomming_message_result.unwrap();
                    println!("Receive message from cloud");
                }
            }
        }    
    });
}