extern crate lazy_static;

mod protos;
mod comboard;
mod socket;
mod id;
mod mainboardstate;
mod modulestate;
mod store;

use std::sync::{Mutex, Arc, mpsc::channel};
use comboard::imple;

#[tokio::main]
async fn main() {

    let conn_database = Arc::new(Mutex::new(store::database::init()));

    let d = comboard::get_comboard_client();

    let (_sender_config, receiver_config) = channel::<imple::interface::Module_Config>();
    let (sender_state, receiver_state) = channel::<imple::interface::ModuleStateChangeEvent>();
    let (sender_value, receiver_value) = channel::<imple::interface::ModuleValueValidationEvent>();

    let (sender_socket, receiver_socket) = channel::<(String, Box<dyn modulestate::interface::ModuleValueParsable>)>();

    let sender_socket_hello = sender_socket.clone();

    let comboard_task = d.run(
        comboard::imple::interface::ComboardClientConfig{
        receiver_config: Arc::new(Mutex::new(receiver_config)),
        sender_state_change: Arc::new(Mutex::new(sender_state)),
        sender_value_validation: Arc::new(Mutex::new(sender_value)),
    });

    let module_state_task = modulestate::module_state_task(
        receiver_state,
        receiver_value,
        sender_socket,
        conn_database.clone(),
    );

    let socket_task = socket::socket_task(
        Arc::new(Mutex::new(receiver_socket)),
    );

    mainboardstate::hello_world::task_hello_world(
        sender_socket_hello,
    ).await;

    let (_,_,_) = tokio::join!(
        comboard_task,
        module_state_task,
        socket_task
    );
}
