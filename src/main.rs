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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub mod router;
use router::Router;

fn server_start(interrupt: Arc<AtomicBool>) -> std::thread::JoinHandle<Result<(), IoError>> {
    std::thread::Builder::new()
    .name(String::from("WebServer"))
    .spawn(move || {
        use futures::Stream;        
        
        let s_addr = "127.0.0.1:3000";
        info!("WebServer thread start listening {}...", s_addr);

        let address = s_addr.parse().unwrap();
        let backlog = 1024;
        let tcp_keepalive = Some(Duration::from_secs(300));

        let net_listener = net2::TcpBuilder::new_v4().unwrap()
            //.reuse_port(true).unwrap()
            .bind(address).unwrap()
            .listen(backlog).unwrap();
        
        net_listener.set_nonblocking(true).unwrap();

        let mut core = tokio_core::reactor::Core::new().unwrap();
        let handle = core.handle();
        let core_listener = tokio_core::net::TcpListener::from_listener(net_listener, &address, &handle).unwrap();        
        
        core.run(core_listener.incoming().for_each(move |(stream, socket_addr)| {
            stream.set_keepalive(tcp_keepalive).unwrap();
            info!("Connection from {}", socket_addr);
            if interrupt.load(Ordering::Relaxed) {
                info!("Webserver thread is going to shutdown...");
                return Err(IoError::from(ErrorKind::Interrupted))
            }

            hyper::server::Http::new()
                .keep_alive(true)
                .bind_connection(&handle, stream, socket_addr, Router::new());
              
            Ok(())
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

    let interrupt_handle = Arc::new(AtomicBool::new(false));
    let join_handle = server_start(interrupt_handle.clone());

    std::thread::sleep(Duration::from_secs(10));
    interrupt_handle.store(true, Ordering::Relaxed);
    info!("Interrupt turned");
    drop(std::net::TcpStream::connect("127.0.0.1:3000").unwrap());
    info!("---");

    match join_handle.join().unwrap() {
        Ok(()) => info!("Webserver thread has been successfully completed"),
        Err(e) => {
            if e.kind() == ErrorKind::Interrupted { 
                info!("Webserver thread has been interrupted");
            } else {
                error!("WebServer thread raised an error: {:?}", e);
            }
        }
    }
}
