//! Middleware types.
use crate::endpoint::DynEndpoint;
use hyper::{Body, Request, Response};
use route_recognizer::Params;
use std::sync::Arc;
// use crate::{Request, Response};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// Middleware that wraps around the remaining middleware chain.
#[async_trait]
pub trait Middleware<State>: Send + Sync + 'static {
    /// Asynchronously handle the request, and return a response.
    async fn handle(
        &self,
        state: State,
        request: Request<Body>,
        route_params: Vec<Params>,
        next: Next<'_, State>,
    ) -> Response<Body>;

    /// Set the middleware's name. By default it uses the type signature.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

#[async_trait]
impl<State, F> Middleware<State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(
            State,
            Request<Body>,
            Vec<Params>,
            Next<'a, State>,
        ) -> Pin<Box<dyn Future<Output = Response<Body>> + 'a + Send>>,
{
    async fn handle(
        &self,
        state: State,
        req: Request<Body>,
        route_params: Vec<Params>,
        next: Next<'_, State>,
    ) -> Response<Body> {
        (self)(state, req, route_params, next).await
    }
}

pub struct Next<'a, State> {
    pub(crate) endpoint: &'a DynEndpoint<State>,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware<State>>],
}

impl<State: Clone + Send + Sync + 'static> Next<'_, State> {
    /// Asynchronously execute the remaining middleware chain.
    pub async fn run(
        mut self,
        state: State,
        req: Request<Body>,
        route_params: Vec<Params>,
    ) -> Response<Body> {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(state, req, route_params, self).await
        } else {
            self.endpoint.call(state, req, route_params).await
        }
    }
}
