#[macro_use]
extern crate rocket;

use rocket::State;
use rocket_sync_db_pools::{database, diesel};
use stopwatch::Stopwatch;

mod stopwatch;

#[database("sqlite_timers")]
struct TimersDbConn(diesel::SqliteConnection);

#[get("/")]
fn elapsed(conn: TimersDbConn, stopwatch: &State<Stopwatch>) -> String {
    humantime::format_duration(stopwatch.elapsed_seconds()).to_string()
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .manage(Stopwatch::new_and_start())
        .attach(TimersDbConn::fairing())
        .mount("/", routes![elapsed])
        .launch()
        .await;
}
