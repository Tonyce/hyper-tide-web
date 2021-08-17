use async_trait::async_trait;
use futures::Future;

use crate::middleware::{Middleware, Next};
use hyper::{Body, Request, Response};
use route_recognizer::Params;
use std::sync::Arc;

#[async_trait]
pub trait Endpoint<State: Clone + Send + Sync + 'static>: Send + Sync + 'static {
    async fn call(
        &self,
        state: State,
        req: Request<Body>,
        route_params: Vec<Params>,
    ) -> Response<Body>;
}

pub type DynEndpoint<State> = dyn Endpoint<State>;

#[async_trait]
impl<State, F, Fut> Endpoint<State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(State, Request<Body>, Vec<Params>) -> Fut,
    Fut: Future<Output = Response<Body>> + Send + 'static,
{
    async fn call(
        &self,
        state: State,
        req: Request<Body>,
        route_params: Vec<Params>,
    ) -> Response<Body> {
        let fut = (self)(state, req, route_params);
        fut.await
    }
}

pub(crate) struct MiddlewareEndpoint<E, State> {
    endpoint: E,
    middleware: Vec<Arc<dyn Middleware<State>>>,
}

impl<E, State> MiddlewareEndpoint<E, State>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State>,
{
    pub(crate) fn wrap_with_middleware(
        ep: E,
        middleware: &[Arc<dyn Middleware<State>>],
    ) -> Box<dyn Endpoint<State> + Send + Sync + 'static> {
        if middleware.is_empty() {
            Box::new(ep)
        } else {
            Box::new(Self {
                endpoint: ep,
                middleware: middleware.to_vec(),
            })
        }
    }
}

#[async_trait]
impl<E, State> Endpoint<State> for MiddlewareEndpoint<E, State>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State>,
{
    async fn call(
        &self,
        state: State,
        req: Request<Body>,
        route_params: Vec<Params>,
    ) -> Response<Body> {
        let next = Next {
            endpoint: &self.endpoint,
            next_middleware: &self.middleware,
        };
        next.run(state, req, route_params).await
    }
}
