use std::{time::SystemTime, fs::OpenOptions};
use std::io::stdout;

pub fn init_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        // .chain(
        //     fern::Dispatch::new()
        //     .format(|out, message, record| {
        //         out.finish(format_args!(
        //             "[{} {} {}] {}",
        //             humantime::format_rfc3339_seconds(SystemTime::now()),
        //             record.level(),
        //             record.target(),
        //             message
        //         ))
        //     })
        //     .chain(
        //         fern::Dispatch::new()
        //         .level(log::LevelFilter::Trace)
        //         .filter(|metadata| metadata.level() == log::LevelFilter::Error)
        //         .chain(stdout())
        //     )
        //     // .level(log::LevelFilter::Debug)
        //     // .chain(
        //     //     OpenOptions::new()
        //     //     .write(true)
        //     //     .create(true)
        //     //     .append(false)
        //     //     .open("output.log")?
        //     // )
        // )
        // .chain(
        //     fern::Dispatch::new()
        //     .level(log::LevelFilter::Trace)
        //     .filter(|metadata| metadata.level() == log::LevelFilter::Trace)
        //     .chain(
        //         OpenOptions::new()
        //         .create(true)
        //         .write(true)
        //         .append(false)
        //         // .create_new(true)
        //         .open("cputrace.log")?
        //     )
        // )
        .apply()?;
    
    return Ok(());
}
