#[macro_use]
extern crate log;
extern crate env_logger;
extern crate time;

extern crate futures;
extern crate hyper;

use hyper::server;

struct Router {
    counter: i32,
}

impl Router {
    fn new() -> Router {
        Router { counter: 0 }
    }
}

impl server::Service for Router {
    type Request = server::Request;
    type Response = server::Response;
    type Error = hyper::Error;
    type Future = Box<futures::Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _req: Self::Request) -> Self::Future {
        //self.counter+=1;

        let mut response = Self::Response::new();
        response.set_body(self.counter.to_string());

        let result = futures::future::ok(response);

        Box::new(result)
    }
}

fn logger_setup() {
    use std::env;
    use log::LogLevelFilter;
    use env_logger::{LogBuilder, LogTarget};

    let mut builder = LogBuilder::new();
    builder.target(LogTarget::Stdout);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    } else {
        builder.filter(None, LogLevelFilter::Trace);
        builder.format(|record| {
            let location = record.location();
            format!(
                "{} ({}) [{}] {} <module {}, file {}:{}>", 
                time::now().rfc3339(), 
                record.target(),
                record.level(),
                record.args(),
                location.module_path(),
                location.file(),
                location.line()
            )
        });
    }

    builder.init().unwrap();
}

fn main() {
    logger_setup();


    trace!("trace");
    debug!("debug");
    warn!("warning");

    info!("Starting up...");  
    error!("error");
    
}
