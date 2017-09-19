use futures::{future, Future};
use hyper::{server, Error};

pub struct Router {
    counter: i32,
}

impl Router {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl server::Service for Router {
    type Request = server::Request;
    type Response = server::Response;
    type Error = Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _req: Self::Request) -> Self::Future {
        //self.counter+=1;

        let mut response = Self::Response::new();
        response.set_body(self.counter.to_string());

        let result = future::ok(response);

        Box::new(result)
    }
}