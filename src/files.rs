use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};

use notify::event::{AccessKind, AccessMode, ModifyKind, RenameMode};
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::info_span;

pub fn read_file<P, D>(path: P) -> Result<D, io::Error>
where
    P: AsRef<Path>,
    D: DeserializeOwned,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let app_data = serde_json::from_reader(reader)?;
    Ok(app_data)
}

pub fn write_file<P, S>(buf: P, value: &S) -> Result<(), io::Error>
where
    P: AsRef<Path>,
    S: ?Sized + Serialize,
{
    let parent_directory = buf.as_ref().parent().unwrap();
    fs::create_dir_all(parent_directory)?;
    let file = File::create(buf)?;

    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.flush()?;
    Ok(())
}

#[must_use]
pub fn watch_file<P, F>(path: P, handler: F) -> RecommendedWatcher
where
    P: Into<PathBuf>,
    F: 'static + Fn() + Send,
{
    let watched_path = path.into();

    let span = info_span!(
        "watch_state_file",
        path = watched_path
            .canonicalize()
            .as_ref()
            .unwrap_or(&watched_path)
            .to_string_lossy()
            .into_owned()
    );
    let _enter = span.clone().entered();

    let parent = watched_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .unwrap_or_else(|_| panic!("Parent path for {} does not exist.", watched_path.display()));
    let mut watcher = recommended_watcher(move |event| {
        if let Ok(Event {
            kind:
                EventKind::Access(AccessKind::Close(AccessMode::Write))
                | EventKind::Modify(ModifyKind::Name(RenameMode::To)),
            paths,
            ..
        }) = event
        {
            for path in paths {
                if let Ok(path) = path.canonicalize() {
                    if Some(path) == watched_path.canonicalize().ok() {
                        span.in_scope(|| {
                            tracing::debug!("loading state from file");
                        });
                        handler();
                    }
                }
            }
        }
    })
    .unwrap();
    watcher.watch(&parent, RecursiveMode::NonRecursive).unwrap();

    tracing::debug!("started monitoring of state file");

    watcher
}
