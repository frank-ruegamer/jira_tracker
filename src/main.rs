#[macro_use]
extern crate rocket;

use rocket::State;
use stopwatch::Stopwatch;

mod stopwatch;

#[get("/")]
fn elapsed(stopwatch: &State<Stopwatch>) -> String {
    stopwatch.elapsed().as_secs().to_string()
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .manage(Stopwatch::new_and_start())
        .mount("/", routes![elapsed])
        .launch()
        .await;
}
