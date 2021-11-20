
use warp::Filter;



pub fn get_server() -> tokio::task::JoinHandle<()> {

    let routes = warp::path("hello")
    .and(warp::path::param())
    .and(warp::header("user-agent"))
    .map(|param: String, agent: String| {
        format!("Hello {}, whose agent is {}", param, agent)
    });
    
    return tokio::spawn( async move {
        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
    });
}