use crate::{AppData, Status};
use rocket::response::Responder;
use rocket::{Request, Response};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::{env, fs, io};

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

pub fn read_state_file() -> Result<AppData, io::Error> {
    let file_name = env::var(JSON_FILE).unwrap();
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    let app_data = serde_json::from_reader(reader)?;
    Ok(app_data)
}

pub fn write_state_file(app_data: &AppData) -> Result<(), io::Error> {
    let file_name = env::var(JSON_FILE).unwrap();
    let file_path = Path::new(&file_name);
    let parent_directory = file_path.parent().unwrap();
    fs::create_dir_all(parent_directory)?;
    let file = File::create(file_path)?;

    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, app_data)?;
    writer.flush()?;
    Ok(())
}
