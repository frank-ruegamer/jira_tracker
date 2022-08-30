use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, io};

use hotwatch::Hotwatch;
use rocket::response::Responder;
use rocket::{Request, Response};
use serde::de::DeserializeOwned;

use crate::{AppData, DebouncedEvent, Status};

pub const JIRA_ACCOUNT_ID: &str = "JIRA_ACCOUNT_ID";
pub const TEMPO_API_TOKEN: &str = "TEMPO_API_TOKEN";

const JSON_FILE: &str = "JSON_FILE";

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
    let file = File::open(get_config_file())?;
    let reader = BufReader::new(file);
    let app_data = serde_json::from_reader(reader)?;
    Ok(app_data)
}

pub fn write_state_file(app_data: &AppData) -> Result<(), io::Error> {
    let file_path = get_config_file();
    let parent_directory = file_path.parent().unwrap();
    fs::create_dir_all(parent_directory)?;
    let file = File::create(file_path)?;

    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, app_data)?;
    writer.flush()?;
    Ok(())
}

pub fn watch_config_file<F>(f: F) -> Hotwatch
where
    F: 'static + Fn() + Send,
{
    let watched_path = get_config_file();
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

fn get_config_file() -> PathBuf {
    let file_name = env::var(JSON_FILE).unwrap();
    let expanded_path = shellexpand::full(&file_name).unwrap();
    PathBuf::from(expanded_path.into_owned())
}
