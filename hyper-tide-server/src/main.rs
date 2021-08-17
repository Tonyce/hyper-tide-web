use hyper_tide::{http, Body, Next, Params, Request, Response, Server};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

mod route;

#[tokio::main]
async fn main() {
    let mut state = HashMap::new();
    state.insert("Daniel".to_string(), "798-1364".to_string());
    let state = Arc::new(state);

    let addr = "127.0.0.1:8989"
        .parse()
        .expect("Unable to parse socket address");

    // app.with(test_middleware);
    let mut app = Server::with_state(state.clone());
    app.at("/").nest(route::index(state.clone()));
    app.at("/api").with(test_middleware).nest(route::api());

    println!("Listening on http://{}", addr);
    app.listen(&addr).await.unwrap();
}

fn test_middleware<'a, State: Clone + Send + Sync + 'static>(
    state: State,
    mut request: Request<Body>,
    route_params: Vec<Params>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'a>> {
    Box::pin(async {
        println!("middleware");
        // "ok".to_owned();
        // println!("{:?}", result);
        // if false {
        request.extensions_mut().insert("hello middleware");
        let mut response = next.run(state, request, route_params).await;
        let header = response.headers_mut();
        header.insert("key", http::HeaderValue::from_str("middlewrae").unwrap());
        response

        // } else {
        // let body: Vec<u8> = "hellowrold middleware\n".to_owned().into_bytes();
        // http::Response::builder()
        // .status(http::StatusCode::NOT_ACCEPTABLE)
        // .body(body)
        // .unwrap()
        // }
    })
}

fn test_middleware_2<'a, State: Clone + Send + Sync + 'static>(
    state: State,
    mut request: Request<Body>,
    route_params: Vec<Params>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'a>> {
    Box::pin(async {
        println!("middleware2 {}", request.method());
        let result = request.extensions_mut().insert("hello middleware2");
        next.run(state, request, route_params).await
        // if let Some(user) = request.state().find_user().await {
        //     tide::log::trace!("user loaded", {user: user.name});
        //     request.set_ext(user);
        //     Ok(next.run(request).await)
        // // this middleware only needs to run before the endpoint, so
        // // it just passes through the result of Next
        // } else {
        //     // do not run endpoints, we could not find a user
        //     Ok(Response::new(StatusCode::Unauthorized))
        // }
    })
}
