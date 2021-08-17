use crate::endpoint::{Endpoint, MiddlewareEndpoint};
use crate::middleware::Middleware;
use crate::router::Router;
use hyper::http::{self};
use hyper::{Body, Request, Response};
use route_recognizer::Params;
use std::sync::Arc;

pub struct Route<'a, State> {
    router: &'a mut Router<State>,
    path: String,
    middleware: Vec<Arc<dyn Middleware<State>>>,
    prefix: bool,
}

impl<'a, State: Clone + Send + Sync + 'static> Route<'a, State> {
    pub(crate) fn new(router: &'a mut Router<State>, path: String) -> Self {
        Self {
            router,
            path,
            middleware: Vec::new(),
            prefix: false,
        }
    }

    pub fn method(&mut self, method: http::Method, ep: impl Endpoint<State>) -> &mut Self {
        // self.router.add(&self.path, method, Box::new(ep));
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);

            self.router.add(
                &self.path,
                method.clone(),
                MiddlewareEndpoint::wrap_with_middleware(ep.clone(), &self.middleware),
            );
            let wildcard = self.at("*--tide-path-rest");
            wildcard.router.add(
                &wildcard.path,
                method,
                MiddlewareEndpoint::wrap_with_middleware(ep, &wildcard.middleware),
            );
        } else {
            self.router.add(
                &self.path,
                method,
                MiddlewareEndpoint::wrap_with_middleware(ep, &self.middleware),
            );
        }
        self
    }

    pub fn get(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::GET, ep);
        self
    }

    pub fn head(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::HEAD, ep);
        self
    }

    /// Add an endpoint for `PUT` requests
    pub fn put(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::PUT, ep);
        self
    }

    /// Add an endpoint for `POST` requests
    pub fn post(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::POST, ep);
        self
    }

    /// Add an endpoint for `DELETE` requests
    pub fn delete(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::DELETE, ep);
        self
    }

    /// Add an endpoint for `OPTIONS` requests
    pub fn options(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::OPTIONS, ep);
        self
    }

    /// Add an endpoint for `CONNECT` requests
    pub fn connect(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::CONNECT, ep);
        self
    }

    /// Add an endpoint for `PATCH` requests
    pub fn patch(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::PATCH, ep);
        self
    }

    /// Add an endpoint for `TRACE` requests
    pub fn trace(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::TRACE, ep);
        self
    }

    /// Apply the given middleware to the current route.
    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        // println!(
        //     "Adding middleware {} to route {:?}",
        //     middleware.name(),
        //     self.path
        // );
        self.middleware.push(Arc::new(middleware));
        self
    }

    pub fn reset_middleware(&mut self) -> &mut Self {
        self.middleware.clear();
        self
    }

    /// Extend the route with the given `path`.
    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b, State> {
        let mut p = self.path.clone();

        if !p.ends_with('/') && !path.starts_with('/') {
            p.push('/');
        }

        if path != "/" {
            p.push_str(path);
        }

        Route {
            router: &mut self.router,
            path: p,
            middleware: self.middleware.clone(),
            prefix: false,
        }
    }

    pub fn all(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);

            self.router.add_all(
                &self.path,
                MiddlewareEndpoint::wrap_with_middleware(ep.clone(), &self.middleware),
            );
            let wildcard = self.at("*--tide-path-rest");
            wildcard.router.add_all(
                &wildcard.path,
                MiddlewareEndpoint::wrap_with_middleware(ep, &wildcard.middleware),
            );
        } else {
            self.router.add_all(
                &self.path,
                MiddlewareEndpoint::wrap_with_middleware(ep, &self.middleware),
            );
        }
        self
    }

    pub fn nest<InnerState>(&mut self, service: crate::Server<InnerState>) -> &mut Self
    where
        InnerState: Clone + Send + Sync + 'static,
    {
        let prefix = self.prefix;

        self.prefix = true;
        self.all(service);
        self.prefix = prefix;

        self
    }
}

#[derive(Debug)]
struct StripPrefixEndpoint<E>(std::sync::Arc<E>);

impl<E> StripPrefixEndpoint<E> {
    fn new(ep: E) -> Self {
        Self(std::sync::Arc::new(ep))
    }
}

impl<E> Clone for StripPrefixEndpoint<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait::async_trait]
impl<State, E> Endpoint<State> for StripPrefixEndpoint<E>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State>,
{
    async fn call(
        &self,
        state: State,
        mut req: Request<Body>,
        route_params: Vec<Params>,
    ) -> Response<Body> {
        let rest = route_params
            .last()
            .and_then(|params| params.find("--tide-path-rest"))
            .unwrap_or("");

        // TODO @ttang /
        *req.uri_mut() = ("/".to_owned() + rest).parse().unwrap();

        self.0.call(state, req, route_params).await
    }
}
