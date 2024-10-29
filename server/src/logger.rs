use std::{
    ffi::OsStr,
    fs,
    io::{Error as ErrorIO, ErrorKind},
    path::PathBuf,
    str::FromStr,
};

use anyhow::Ok;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::*;

use super::{info, Context, Result};

pub enum Scope {
    Local,
    #[allow(dead_code)]
    Global,
}

pub fn initialize(logger_type: &Scope, logger_level: &str) -> Result<WorkerGuard> {
    let level = get_level(logger_level)?;
    let name = get_binary_name()?;
    let path: PathBuf = match logger_type {
        Scope::Local => {
            let mut path = dirs::data_local_dir().context("Failed to get local data directory")?;
            path.push("log");
            path.push(name.clone());
            path
        }
        Scope::Global => {
            let mut path = PathBuf::from("/var/log");
            path.push(name.clone());
            path
        }
    };
    fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    let file_appender = tracing_appender::rolling::hourly(path.clone(), "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(level)),
        )
        .with(
            tracing_subscriber::fmt::Layer::new()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(level)),
        );

    tracing::subscriber::set_global_default(subscriber).context("Failed to set subscriber")?;
    info!("{} logging files are set at: {:?}", name, path);
    Ok(guard)
}

fn get_binary_name() -> Result<String> {
    let binary_path = std::env::current_exe()
        .map_err(|_| ErrorIO::new(ErrorKind::Other, "couldn't get binary path"))?;
    let binary_name = binary_path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| ErrorIO::new(ErrorKind::Other, "couldn't get binary name"))?;
    Ok(binary_name.to_owned())
}

fn get_level(level: &str) -> Result<Level> {
    Level::from_str(level).context(format!("Invalid log level: {level}"))
}
