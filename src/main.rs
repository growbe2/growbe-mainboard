#![warn(const_err)]
extern crate lazy_static;

mod protos;
mod comboard;
mod socket;
mod logger;
mod id;
mod mainboardstate;
mod modulestate;
mod store;
mod plateform;
mod utils;
mod server;
mod cmd;

use std::sync::{Mutex, Arc, mpsc::channel};

use crate::{comboard::imple::channel::{ComboardConfigChannelManager, ComboardAddr}, protos::board::RunningComboard};

#[tokio::main(flavor = "multi_thread")]
async fn main() {

    if let Some(()) = cmd::handle_command_line_arguments() {
        return;
    }

    logger::setup_log();

    log::info!("starting mainboard with id {}", id::get());

    // Initializing database
    let conn_database = Arc::new(Mutex::new(store::database::init()));

    let boards = comboard::get_comboard_client();

    let (sender_socket, receiver_socket) = channel::<(String, Box<dyn modulestate::interface::ModuleValueParsable>)>();

    let sender_socket_hello = sender_socket.clone();
    let sender_socket_localconnection = sender_socket.clone();


    let mut config_channel_manager = ComboardConfigChannelManager::new();

    // Create the task to run the comboard
    boards.iter().for_each(|(info, board)| {
        let receiver = config_channel_manager.create_channel(ComboardAddr{ imple: info.imple.clone(), addr: info.addr.clone()});
        board.run(receiver);
    });

    // Create the task to handle the modules state 
    let module_state_task = modulestate::module_state_task(
        sender_socket,
        modulestate::store::ModuleStateStore::new(conn_database.clone()),
        config_channel_manager.get_reference(),
        modulestate::alarm::store::ModuleAlarmStore::new(conn_database.clone()),
    );

    // Create the task for the communication socket from outside the app
    let socket_task = socket::socket_task(
        Arc::new(Mutex::new(receiver_socket)),
        &mainboardstate::config::CONFIG.mqtt,
    );


    // Run the hello world task to start the application
    mainboardstate::hello_world::task_hello_world(
        sender_socket_hello,
        boards.iter().map(|x| RunningComboard {addr: x.0.addr.clone(), imple: x.0.imple.clone(), ..Default::default()}).collect(),
    ).await;

    mainboardstate::localconnection::task_local_connection(
        sender_socket_localconnection,
    ).await;



    #[cfg(feature = "http_server")]
    let server_task = server::http::get_server(&crate::mainboardstate::config::CONFIG.server);
    #[cfg(not(feature = "http_server"))]
    let server_task  = async {};


    let _ = tokio::join!(server_task, module_state_task, socket_task);
}
