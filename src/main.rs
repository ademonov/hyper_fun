#[macro_use]
extern crate log;
extern crate env_logger;
extern crate time;

extern crate net2;
extern crate tokio_core;

extern crate futures;
extern crate hyper;

use std::io::Error as IoError;
use std::io::ErrorKind;
use hyper::server;

struct Router {
    counter: i32,
}

impl Router {
    fn new() -> Self {
        Self { counter: 0 }
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

fn server_start() -> std::thread::JoinHandle<std::result::Result<(), IoError>> {
    std::thread::Builder::new()
    .name(String::from("WebServer"))
    .spawn(move || {
        use futures::Stream;
        use std::time::Duration;
        
        let s_addr = "127.0.0.1:3000";
        info!("WebServer thread start listening {}...", s_addr);

        let address = s_addr.parse().unwrap();
        let backlog = 1024;
        let net_listener = net2::TcpBuilder::new_v4().unwrap()
            //.reuse_port(true).unwrap()
            .bind(address).unwrap()
            .listen(backlog).unwrap();
        
        net_listener.set_nonblocking(true).unwrap();

        let mut core = tokio_core::reactor::Core::new().unwrap();
        let handle = core.handle();
        let core_listener = tokio_core::net::TcpListener::from_listener(net_listener, &address, &handle).unwrap();        
        
        core.run(core_listener.incoming().for_each(move |(stream, socket_addr)| {
            stream.set_keepalive(Some(Duration::from_secs(300))).unwrap();
            info!("Connection from {}", socket_addr);
            hyper::server::Http::new()
                .keep_alive(true)
                .bind_connection(&handle, stream, socket_addr, Router::new());
            //Ok(())
            
            Err(IoError::from(ErrorKind::Interrupted))
        }))
    }).unwrap()
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
        builder.filter(Some("hyper_fun"), LogLevelFilter::Trace);
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
    info!("Starting up...");  

    let join_handle = server_start();
    match join_handle.join().unwrap() {
        Ok(()) => info!("Webserver thread was successfully completed"),
        Err(e) => {
            if e.kind() == ErrorKind::Interrupted { 
                info!("Webserver thread was interrupted");
            } else {
                error!("WebServer thread raised an error: {:?}", e);
            }
        }
    }
}
