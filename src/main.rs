#[macro_use]
extern crate rocket;

use rocket::State;
use stopwatch::Stopwatch;

mod instant_serializer;
mod stopwatch;

#[get("/")]
fn elapsed(stopwatch: &State<Stopwatch>) -> String {
    humantime::format_duration(stopwatch.elapsed_seconds()).to_string()
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .manage(Stopwatch::new_and_start())
        .mount("/", routes![elapsed])
        .launch()
        .await;
}
