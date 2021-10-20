#[macro_use] extern crate rocket;

use rocket::State;
use stopwatch::Stopwatch;

mod stopwatch;

static ID = Atomic

#[get("/")]
fn elapsed(stopwatch: &State<Stopwatch>) -> String {
    stopwatch.elapsed().as_secs().to_string()
}

#[launch]
fn rocket() -> _ {
    rocket::build().manage(Stopwatch::new_and_start()).mount("/", routes![elapsed])
}