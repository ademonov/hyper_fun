use time;
use std::env;
use log::LogLevelFilter;
use env_logger::{LogBuilder, LogTarget};

pub fn setup() {
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