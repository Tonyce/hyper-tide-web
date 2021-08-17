use crate::endpoint::DynEndpoint;
use hyper::{Body, Method, Request, Response, StatusCode};
use route_recognizer::{Params, Router as MethodRouter};
use std::collections::HashMap;
pub struct Router<State> {
    pub method_map: HashMap<hyper::http::Method, MethodRouter<Box<DynEndpoint<State>>>>,
    all_method_router: MethodRouter<Box<DynEndpoint<State>>>,
}

pub(crate) struct Selection<'a, State> {
    pub(crate) endpoint: &'a DynEndpoint<State>,
    pub(crate) params: Params,
}

impl<State: Clone + Send + Sync + 'static> Router<State> {
    pub fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            all_method_router: MethodRouter::new(),
            // all_method_router: "state".to_string(),
        }
    }

    pub(crate) fn add(&mut self, path: &str, method: Method, ep: Box<DynEndpoint<State>>) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, ep)
    }

    pub(crate) fn route(&self, path: &str, method: &hyper::Method) -> Selection<'_, State> {
        if let Some(m) = self
            .method_map
            .get(method)
            .and_then(|r| r.recognize(path).ok())
        {
            let handler = m.handler();
            let params = m.params().clone();
            Selection {
                endpoint: &***handler,
                params,
            }
        } else if let Ok(m) = self.all_method_router.recognize(path) {
            let handler = m.handler();
            let params = m.params().clone();
            Selection {
                endpoint: &***handler,
                params,
            }
        } else if method == Method::HEAD {
            self.route(path, &Method::GET)
        } else if self
            .method_map
            .iter()
            .filter(|(k, _)| **k != method)
            .any(|(_, r)| r.recognize(path).is_ok())
        {
            // If this `path` can be handled by a callback registered with a different HTTP method
            // should return 405 Method Not Allowed
            Selection {
                endpoint: &method_not_allowed,
                params: Params::new(),
            }
        } else {
            Selection {
                endpoint: &not_found_endpoint,
                params: Params::new(),
            }
        }
    }

    pub(crate) fn add_all(&mut self, path: &str, ep: Box<DynEndpoint<State>>) {
        self.all_method_router.add(path, ep)
    }
}

async fn not_found_endpoint<State: Clone + Send + Sync + 'static>(
    _state: State,
    _req: Request<Body>,
    _route_params: Vec<Params>,
) -> Response<Body> {
    // "Not Found".to_owned()
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
    // Ok(Response::new(StatusCode::NotFound))
}

async fn method_not_allowed<State: Clone + Send + Sync + 'static>(
    _state: State,
    _req: Request<Body>,
    _route_params: Vec<Params>,
) -> Response<Body> {
    // "Method Not Allowed".to_owned()
    // Ok(Response::new(StatusCode::NotFound))
    let response = Response::builder();
    let response = response.status(StatusCode::NOT_IMPLEMENTED);
    response.body(Body::empty()).unwrap()
}
