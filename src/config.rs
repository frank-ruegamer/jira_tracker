use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, io};

use hotwatch::notify::DebouncedEvent;
use hotwatch::Hotwatch;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::{Request, Response};
use serde::de::DeserializeOwned;
use static_init::dynamic;

use crate::app_data::AppData;

fn env_var<K: Display + AsRef<OsStr>>(key: K) -> String {
    env::var(&key).unwrap_or_else(|_| panic!("env var `{}` not set", key))
}

#[dynamic]
pub static JIRA_ACCOUNT_ID: String = env_var("JIRA_ACCOUNT_ID");
#[dynamic]
pub static TEMPO_API_TOKEN: String = env_var("TEMPO_API_TOKEN");
#[dynamic]
static JSON_FILE: String = shellexpand::full(&env_var("JSON_FILE"))
    .unwrap()
    .into_owned();

pub struct LogError(pub Box<dyn Error>);

impl<'r, 'o: 'r> Responder<'r, 'o> for LogError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'o> {
        println!("Internal Server Error: {}", self.0);
        Response::build().status(Status::InternalServerError).ok()
    }
}

pub fn get_initial_state() -> AppData {
    read_state_file().unwrap_or_else(|_| AppData::new())
}

pub fn read_state_file<F>() -> Result<F, io::Error>
where
    F: DeserializeOwned,
{
    let file = File::open(get_state_file())?;
    let reader = BufReader::new(file);
    let app_data = serde_json::from_reader(reader)?;
    Ok(app_data)
}

pub fn write_state_file(app_data: &AppData) -> Result<(), io::Error> {
    let file_path = get_state_file();
    let parent_directory = file_path.parent().unwrap();
    fs::create_dir_all(parent_directory)?;
    let file = File::create(file_path)?;

    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, app_data)?;
    writer.flush()?;
    Ok(())
}

pub fn watch_state_file<F>(f: F) -> Hotwatch
where
    F: 'static + Fn() + Send,
{
    let watched_path = get_state_file();
    let mut hotwatch = Hotwatch::new_with_custom_delay(Duration::from_secs(1)).unwrap();
    hotwatch
        .watch(
            watched_path.canonicalize().unwrap().parent().unwrap(),
            move |event| {
                if let DebouncedEvent::Write(ref event_path) = event {
                    if event_path.canonicalize().ok() == watched_path.canonicalize().ok() {
                        f();
                    }
                }
            },
        )
        .unwrap();
    hotwatch
}

fn get_state_file() -> PathBuf {
    PathBuf::from(JSON_FILE.deref())
}
