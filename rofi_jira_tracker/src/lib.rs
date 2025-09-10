extern crate core;

use domain::TrackerInformation;
use rofi_mode::cairo::Surface;
use rofi_mode::{export_mode, Action, Api, Event, Matcher};
use std::future::Future;
use std::time::Duration;

export_mode!(Mode);

struct Mode<'rofi> {
    api: Api<'rofi>,
    entries: Vec<Entry>,
}

enum Entry {
    Tracker(TrackerInformation),
    NewTracker,
    Refresh,
    Submit,
}

impl From<&Entry> for rofi_mode::String {
    fn from(value: &Entry) -> Self {
        match value {
            Entry::Tracker(tracker) => (&tracker.key).into(),
            Entry::NewTracker => "Add New".into(),
            Entry::Refresh => "Refresh issues".into(),
            Entry::Submit => "Submit".into(),
        }
    }
}

async fn get() -> String {
    tokio::time::sleep(Duration::from_secs(1)).await;
    "1234".into()
}

impl<'rofi> rofi_mode::Mode<'rofi> for Mode<'rofi> {
    const NAME: &'static str = "jira-tracker\0";

    fn init(api: Api<'rofi>) -> Result<Self, ()> {
        get()
        let response = reqwest::blocking::get("http://localhost:8081/trackers").map_err(|e| {
            eprintln!("{}", e);
            ()
        })?;
        let trackers = response.json::<Vec<TrackerInformation>>().map_err(|e| {
            eprintln!("{}", e);
            ()
        })?;
        let mut entries: Vec<_> = trackers
            .into_iter()
            .map(|tracker| Entry::Tracker(tracker))
            .collect();

        entries.push(Entry::NewTracker);
        entries.push(Entry::Refresh);
        entries.push(Entry::Submit);
        Ok(Mode { api, entries })
    }

    fn entries(&mut self) -> usize {
        self.entries.len()
    }

    fn entry_content(&self, line: usize) -> rofi_mode::String {
        (&self.entries[line]).into()
    }

    fn entry_icon(&mut self, _line: usize, _height: u32) -> Option<Surface> {
        unreachable!()
    }

    fn react(&mut self, _event: Event, _input: &mut rofi_mode::String) -> Action {
        match _event {
            Event::Ok { alt, selected } => {
                println!("alt {}, selected {}", alt, selected);
                match &self.entries[selected] {
                    Entry::Tracker(tracker) => {
                        println!("key {:?}", tracker);
                    }
                    Entry::NewTracker => {}
                    Entry::Refresh => {}
                    Entry::Submit => {}
                }
                Action::Exit
            }
            _ => Action::Exit,
        }
    }

    fn matches(&self, line: usize, matcher: Matcher<'_>) -> bool {
        matcher.matches(self.entry_content(line).as_str())
    }
}
