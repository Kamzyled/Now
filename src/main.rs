#[macro_use] extern crate rocket;

mod routes;

use rocket::fs::FileServer;
use routes::build_rocket;

#[launch]
fn rocket() -> _ {
    // Attach templates, mount routes, serve static if needed later.
    build_rocket()
        .mount("/public", FileServer::from("public")) // optional folder for css later
}
