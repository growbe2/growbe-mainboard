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
mod server;
mod plateform;
mod utils;

use std::sync::{Mutex, Arc, mpsc::channel};

use tokio::select;

#[tokio::main(flavor = "multi_thread")]
async fn main() {

    logger::setup_log();

    log::info!("starting mainboard with id {}", id::get());

    // Initializing database
    let conn_database = Arc::new(Mutex::new(store::database::init()));

    let (boards, running_boards) = comboard::get_comboard_client();

    let (sender_socket, receiver_socket) = channel::<(String, Box<dyn modulestate::interface::ModuleValueParsable>)>();

    let sender_socket_hello = sender_socket.clone();
    let sender_socket_localconnection = sender_socket.clone();

    // Create the task to run the comboard
    boards.iter().for_each(|x| {
        x.run();
    });

    // Create the task to handle the modules state 
    let module_state_task = modulestate::module_state_task(
        sender_socket,
        modulestate::store::ModuleStateStore::new(conn_database.clone()),
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
        running_boards,
    ).await;

    mainboardstate::localconnection::task_local_connection(
        sender_socket_localconnection,
    ).await;



    let server_task = server::http::get_server(&crate::mainboardstate::config::CONFIG.server);

    // Wait for all task to finish (they should never end)
    //let _ret = tokio::join!(
    //    server_task,
    //    module_state_task,
    //    socket_task,
    //);

    select! {
        _ = server_task => {},
        _ = module_state_task => {},
        _ = socket_task => {},
    }

    //let exit_code = if let Err(_) = ret {
    //    1
    //} else {
    //    0
    //};
    //std::process::exit(exit_code);
}
