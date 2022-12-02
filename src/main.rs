extern crate lazy_static;

mod comboard;
mod mainboardstate;
mod modulestate;
mod plateform;
mod protos;
mod server;
mod socket;
mod store;
mod utils;

use std::sync::{mpsc::channel, Arc, Mutex};

use crate::{
    comboard::imple::channel::{ComboardAddr, ComboardConfigChannelManager},
    protos::board::RunningComboard,
};

use crate::mainboardstate::update::autoupdate;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Check if we try to run a command , if stop program to prevent the main thread to start
    if let Some(()) = growbe_shared::cmd::handle_command_line_arguments() {
        return;
    }

    // starting main thread

    growbe_shared::logger::setup_log(&crate::mainboardstate::config::CONFIG.logger);

    log::info!("starting mainboard with id {}", growbe_shared::id::get());

    // try to perform an autoupdate if the feature is started

    autoupdate();

    // Initializing database
    let conn_database = Arc::new(Mutex::new(store::database::init(None)));

    // Get the list of running comboards
    let boards = comboard::get_comboard_client();

    // Create the channel to send the data to the socket thread
    let (sender_socket, receiver_socket) =
        channel::<(String, Box<dyn modulestate::interface::ModuleValueParsable>)>();

    // Create sender copy to give to some starting task
    let sender_socket_hello = sender_socket.clone();
    let sender_socket_localconnection = sender_socket.clone();

    // Create the ComboardConfigChannelManager use to keep the state of all running comboards
    // threads
    let mut config_channel_manager = ComboardConfigChannelManager::new();

    // Create the task to run the comboard
    boards.iter().for_each(|(info, board)| {
        let receiver = config_channel_manager.create_channel(ComboardAddr {
            imple: info.imple.clone(),
            addr: info.addr.clone(),
        });
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
    if let Err(err) = mainboardstate::hello_world::task_hello_world(
        sender_socket_hello,
        boards
            .iter()
            .map(|x| RunningComboard {
                addr: x.0.addr.clone(),
                imple: x.0.imple.clone(),
                ..Default::default()
            })
            .collect(),
    )
    .await { log::error!("task_hello_world {:?}", err); }

    if let Err(err) = mainboardstate::localconnection::task_local_connection(sender_socket_localconnection)
        .await { log::error!("task_local_connection : {:?}", err)}

    #[cfg(feature = "http_server")]
    let server_task = server::http::get_server(&crate::mainboardstate::config::CONFIG.server);
    #[cfg(not(feature = "http_server"))]
    let server_task = async {};

    let _ = tokio::join!(server_task, module_state_task, socket_task);
}
