use std::fs::OpenOptions;
use std::io;

use chrono::Local;
use slog::*;
use slog_scope;
use slog_stream;
use thread_id;

pub fn configure_logger(level: Level) {
    let log_path = "/tmp/rustracer.log";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    let drain = level_filter(level, slog_stream::stream(file, MyFormat));
    let log = Logger::root(drain.fuse(), o!());
    slog_scope::set_global_logger(log);

}

macro_rules! now {
    () => ( Local::now().format("%Y-%m-%d %H:%M:%S%.3f") )
}

struct MyFormat;

impl slog_stream::Format for MyFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &Record,
              _logger_values: &OwnedKeyValueList)
              -> io::Result<()> {
        let msg = format!("{} [{}][{:x}][{}:{}] - {}\n",
                          now!(),
                          rinfo.level(),
                          thread_id::get(),
                          rinfo.file(),
                          rinfo.line(),
                          rinfo.msg());
        try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}
