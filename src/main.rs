#![feature(trait_upcasting)]
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

use std::sync::{Arc, Mutex};

use growbe_shared::init_tracing;
use tokio::sync::mpsc::channel;

use crate::{
    comboard::imple::channel::{ComboardAddr, ComboardConfigChannelManager},
    protos::board::RunningComboard,
};

use crate::mainboardstate::update::autoupdate;

use crate::comboard::imple::virt::create_virtual_comboard_cmd;
use crate::modulestate::interface::ModuleMsg;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Check if we try to run a command , if stop program to prevent the main thread to start
    if let Some(()) = growbe_shared::cmd::handle_command_line_arguments() {
        return;
    }

    init_tracing();

    #[cfg(not(feature = "debug"))]
    growbe_shared::logger::setup_log(&crate::mainboardstate::config::CONFIG.logger);

    log::info!("starting mainboard with id {}", growbe_shared::id::get());

    // try to perform an autoupdate if the feature is started
    autoupdate();

    // Initializing database
    let conn_database = Arc::new(Mutex::new(store::database::init(None)));

    let (sender_virt, receiver_virt) = create_virtual_comboard_cmd();

    let (sender_module, receiver_module) = channel::<ModuleMsg>(100);

    // Get the list of running comboards
    let mut boards = comboard::get_comboard_client(receiver_virt);

    // Create the channel to send the data to the socket thread
    let (sender_socket, receiver_socket) =
        channel::<crate::socket::ss::SenderPayload>(200);

    // Create sender copy to give to some starting task
    let sender_socket_hello = sender_socket.clone();
    let sender_socket_localconnection = sender_socket.clone();

    // Create the ComboardConfigChannelManager use to keep the state of all running comboards
    // threads
    let mut config_channel_manager = ComboardConfigChannelManager::new();

    // Create the task to run the comboard
    for (info, board) in boards.iter_mut() {
        let receiver = config_channel_manager.create_channel(ComboardAddr {
            imple: info.imple.clone(),
            addr: info.addr.clone(),
        });
        board.run(sender_module.clone(), receiver);
    }

    // Create the task to handle the modules state
    let module_state_task = modulestate::task::module_state_task(
        sender_socket.clone(),
        modulestate::store::ModuleStateStore::new(conn_database.clone()),
        config_channel_manager.get_reference(),
        modulestate::alarm::store::ModuleAlarmStore::new(conn_database.clone()),
        sender_module.clone(),
        receiver_module,
    );

    // Create the task for the communication socket from outside the app
    let socket_task = socket::socket_task(
        receiver_socket,
        sender_virt,
        sender_module,
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
    .await
    {
        log::error!("task_hello_world {:?}", err);
    }

    if let Err(err) =
        mainboardstate::localconnection::task_local_connection(sender_socket_localconnection).await
    {
        log::error!("task_local_connection : {:?}", err)
    }

    #[cfg(feature = "http_server")]
    let server_task = server::http::get_server(&crate::mainboardstate::config::CONFIG.server);
    #[cfg(not(feature = "http_server"))]
    let server_task = async {};

    //let _ = tokio::join!(server_task, socket_task);
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                println!("closing down");
                sender_socket
                    .send((
                        "/disconnecting".into(),
                        Box::new(crate::protos::board::HelloWord::new()),
                    ))
                    .await
                    .unwrap();
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                std::process::exit(0);
            }
            Err(err) => {
                log::error!("unable to listen for shutdown signal {}", err);
            }
        }
    });

    let _ = tokio::join!(server_task, module_state_task, socket_task);

}
