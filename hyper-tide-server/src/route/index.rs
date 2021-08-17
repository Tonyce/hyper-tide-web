use std::fmt::Debug;

use hyper_tide::{http, Body, Next, Params, Request, Response, Server};

pub fn index<State>(state: State) -> Server<State>
where
    State: Clone + Send + Sync + 'static + Debug,
{
    let mut index = Server::with_state(state);

    index.at("/helloworld").get(
        |state: State, _req: Request<Body>, _route_params: Vec<Params>| async move {
            println!("state {:?}", state);
            Response::new(Body::from("helloworld"))
        },
    );

    // index.at("/he/:n").with(test_middleware_2).get(
    index.at("/he/:n").get(
        |_state: State, _req: Request<Body>, route_params: Vec<Params>| async move {
            println!("{:?}", route_params);
            // "hellowrold\n".to_string()
            // let body = "hellowrold\n".to_owned().into_bytes();
            let response = Response::builder().status(http::StatusCode::NOT_FOUND);
            let response = response.header("key", "value").header("contacts", "value");
            response.body(Body::from("hellowrold\n")).unwrap()
        },
    );

    index
}
