use hyper_tide::{http, Body, Next, Params, Request, Response, Server};

pub fn api() -> Server<()> {
    let mut api = Server::with_state(());
    api.at("/hello").get(|state, _, _| async move {
        println!("state {:?}", state);
        Response::new(Body::from("helloworld"))
    });
    api.at("/good/:bye").get(|_, _, route_params| async move {
        println!("route_params {:?}", route_params);
        Response::new(Body::from("goodbye world"))
    });
    api
}
