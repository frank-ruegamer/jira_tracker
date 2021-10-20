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
    let millis = stopwatch.elapsed().as_millis();
    format!("{:.3}", millis as f32 / 1000f32)
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
