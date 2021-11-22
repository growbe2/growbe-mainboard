use serde::{Deserialize, Serialize};
use warp::http::header::{HeaderMap, HeaderValue};
use warp::Filter;


#[derive(Serialize, Deserialize)]
pub struct HttpServerConfig {
    pub addr: String,
    pub port: u16,
}

fn get_default_response_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    headers.insert("Access-Control-Allow-Credentials", HeaderValue::from_static("true"));

    return headers;
}

pub fn get_default_server_config() -> HttpServerConfig {
    return HttpServerConfig{
        addr: String::from("0.0.0.0"),
        port: 3030
    }
}

pub fn get_server(http_server_config: &HttpServerConfig) -> tokio::task::JoinHandle<()> {

    let hello_world = warp::path::end().map(|| "alive");//
    let sys_info = warp::path("sysinfo").map(|| warp::reply::json(&crate::plateform::sysinfo::get_sys_info())).with(warp::reply::with::headers(get_default_response_headers()));
    let process_info = warp::path("process").map(|| warp::reply::json(&crate::plateform::process::get_process_info())).with(warp::reply::with::headers(get_default_response_headers()));
    let net = warp::path("net").map(|| warp::reply::json(&crate::plateform::net::get_net_info())).with(warp::reply::with::headers(get_default_response_headers()));

    let server = warp::serve(hello_world.or(sys_info).or(process_info).or(net));

    let listenning = (http_server_config.addr.parse::<std::net::IpAddr>().unwrap(), http_server_config.port);
    
    return tokio::spawn( async move {
        server.run(listenning).await;
    });
}