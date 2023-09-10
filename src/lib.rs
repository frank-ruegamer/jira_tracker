use crate::domain::TrackerInformation;
use rofi_mode::cairo::Surface;
use rofi_mode::{export_mode, Action, Api, Event, Matcher};

mod domain;

export_mode!(Mode);

struct Mode<'rofi> {
    api: Api<'rofi>,
    trackers: Vec<TrackerInformation>,
}

impl<'rofi> rofi_mode::Mode<'rofi> for Mode<'rofi> {
    const NAME: &'static str = "jira-tracker\0";

    fn init(api: Api<'rofi>) -> Result<Self, ()> {
        let response = reqwest::blocking::get("http://localhost:8080/trackers").map_err(|_| ())?;
        let trackers = response.json::<Vec<TrackerInformation>>().map_err(|_| ())?;
        Ok(Mode { api, trackers })
    }

    fn entries(&mut self) -> usize {
        self.trackers.len()
    }

    fn entry_content(&self, line: usize) -> rofi_mode::String {
        if line < self.trackers.len() {
            self.trackers[line].key.clone().into()
        } else {
            unreachable!()
        }
    }

    fn entry_icon(&mut self, _line: usize, _height: u32) -> Option<Surface> {
        unreachable!()
    }

    fn react(&mut self, _event: Event, _input: &mut rofi_mode::String) -> Action {
        Action::Exit
    }

    fn matches(&self, line: usize, matcher: Matcher<'_>) -> bool {
        matcher.matches(self.entry_content(line).as_str())
    }
}
