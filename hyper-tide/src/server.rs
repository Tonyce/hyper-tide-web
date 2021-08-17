use crate::endpoint::Endpoint;
use crate::middleware::Middleware;
use crate::middleware::Next;
use crate::route::Route;
use crate::router::{Router, Selection};
use hyper::{Body, Error, Request, Response};
use route_recognizer::Params;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Server<State> {
    router: Arc<Router<State>>,
    state: State,
    middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
}

impl Server<()> {
    #[must_use]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl Default for Server<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Server<State>
where
    State: Clone + Send + Sync + 'static,
{
    pub fn with_state(state: State) -> Self {
        Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(vec![
                // #[cfg(feature = "cookies")]
                // Arc::new(cookies::CookiesMiddleware::new()),
                // #[cfg(feature = "logger")]
                // Arc::new(log::LogMiddleware::new()),
            ]),
            state,
        }
    }

    pub async fn listen(self, addr: &SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let make_service = hyper::service::make_service_fn(move |_| {
            let state = self.state.clone();
            let router = self.router.clone();
            let middleware = self.middleware.clone();
            async move {
                Ok::<_, Error>(hyper::service::service_fn(
                    move |request: hyper::Request<Body>| {
                        // let counter = counter.clone();
                        let state = state.clone();
                        let router = router.clone();
                        let middleware = middleware.clone();

                        async move {
                            let path = request.uri().path();
                            let method = request.method();
                            let Selection { endpoint, params } = router.route(path, method);
                            let route_params = vec![params];
                            let next = Next {
                                endpoint,
                                next_middleware: &middleware,
                            };
                            let response = next.run(state, request, route_params).await;
                            Ok::<_, Error>(response)
                        }
                    },
                ))
            }
        });
        let server = hyper::Server::bind(&addr).serve(make_service);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
        Ok(())
    }

    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        let m = Arc::get_mut(&mut self.middleware)
            .expect("Registering middleware is not possible after the Server has started");
        m.push(Arc::new(middleware));
        self
    }

    pub fn at<'a>(&'a mut self, path: &str) -> Route<'a, State> {
        let router = Arc::get_mut(&mut self.router)
            .expect("Registering routes is not possible after the Server has started");
        Route::new(router, path.to_owned())
    }
}

#[async_trait::async_trait]
impl<State: Clone + Sync + Send + 'static, InnerState: Clone + Sync + Send + 'static>
    Endpoint<State> for Server<InnerState>
{
    async fn call(
        &self,
        state: State,
        request: Request<Body>,
        mut route_params: Vec<Params>,
    ) -> Response<Body> {
        // TODO 两个 state
        let _state = state;
        let path = request.uri().path();
        let method = request.method();
        let router = self.router.clone();
        let middleware = self.middleware.clone();
        let state = self.state.clone();

        let Selection { endpoint, params } = router.route(&path, method);
        route_params.push(params);

        let next = Next {
            endpoint,
            next_middleware: &middleware,
        };

        let response = next.run(state, request, route_params).await;
        response
    }
}
