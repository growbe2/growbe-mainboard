pub fn init_tracing() {

    /*
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();

    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        log::error!("failed to subscribe to tracing : {:?}", err);
    } else {
        log::info!("adding tracing to application");
    }
    */

    console_subscriber::init();
}
