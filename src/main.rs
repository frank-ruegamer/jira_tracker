#[macro_use]
extern crate rocket;

use rocket::State;
use stopwatch::Stopwatch;

mod stopwatch;

#[get("/")]
fn elapsed(stopwatch: &State<Stopwatch>) -> String {
    let millis = stopwatch.elapsed().as_millis();
    format!("{:.3}", millis as f32 / 1000f32)
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .manage(Stopwatch::new_and_start())
        .mount("/", routes![elapsed])
        .launch()
        .await;
}
