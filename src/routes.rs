use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rand::{distributions::Alphanumeric, Rng};
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use std::collections::HashMap;
use uuid::Uuid;

// --- Templates attachment ---
pub fn build_rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .manage(APP_STATE.clone())
        .attach(rocket_dyn_templates::Template::fairing())
        .mount(
            "/",
            routes![
                index,
                create_room_get,
                create_room_post,
                join_room_get,
                join_room_post,
                play_get,
                result_get
            ],
        )
}

// --- Models ---
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Player {
    id: String,
    name: String,
    score: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Room {
    code: String,
    players: Vec<Player>,
    current_question_index: usize,
    // later: store answers, challenge progress, etc.
}

#[derive(Clone, Default)]
struct AppState {
    // code -> Room
    rooms: RwLock<HashMap<String, Room>>,
}

static APP_STATE: Lazy<AppState> = Lazy::new(|| AppState {
    rooms: RwLock::new(HashMap::new()),
});

// --- Forms ---
#[derive(FromForm)]
struct CreateRoomForm {
    host_name: String,
}

#[derive(FromForm)]
struct JoinRoomForm {
    code: String,
    name: String,
}

// --- Helpers ---
fn generate_code() -> String {
    // 6-char friendly code, e.g., "A9K4ZT"
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .filter(|c| c.is_ascii_alphanumeric())
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

// --- Routes ---

#[get("/")]
fn index() -> Template {
    Template::render(
        "index",
        context! {
            title: "Welcome Moyosola 💖",
            subtitle: "Created with love by Kamzy 💙",
        },
    )
}

#[get("/create")]
fn create_room_get() -> Template {
    Template::render("create", context! {})
}

#[post("/create", data = "<form>")]
fn create_room_post(form: Form<CreateRoomForm>, state: &State<AppState>) -> Redirect {
    let code = generate_code();
    let host = Player {
        id: Uuid::new_v4().to_string(),
        name: form.host_name.clone(),
        score: 0,
    };
    let room = Room {
        code: code.clone(),
        players: vec![host],
        current_question_index: 0,
    };

    {
        let mut map = state.rooms.write();
        map.insert(code.clone(), room);
    }

    Redirect::to(uri!(join_room_get(code = code)))
}

#[get("/join?<code>")]
fn join_room_get(code: Option<String>) -> Template {
    Template::render(
        "join",
        context! {
            code: code.unwrap_or_default(),
            error: ""
        },
    )
}

#[post("/join", data = "<form>")]
fn join_room_post(form: Form<JoinRoomForm>, state: &State<AppState>) -> Result<Redirect, Status> {
    let mut map = state.rooms.write();
    if let Some(room) = map.get_mut(&form.code) {
        if room.players.len() >= 2 {
            return Err(Status::BadRequest);
        }
        let p = Player {
            id: Uuid::new_v4().to_string(),
            name: form.name.clone(),
            score: 0,
        };
        room.players.push(p);
        Ok(Redirect::to(uri!(play_get(code = form.code.clone()))))
    } else {
        // back to join with error
        Ok(Redirect::to(uri!(join_room_get(Some(format!("{}", form.code))))))
    }
}

#[get("/play/<code>")]
fn play_get(code: String, state: &State<AppState>) -> Template {
    let map = state.rooms.read();
    let maybe_room = map.get(&code);

    // For Step 1 we just show the room and players.
    // In Step 2 we'll load 200 questions and start the flow.
    if let Some(room) = maybe_room {
        let players: Vec<String> = room.players.iter().map(|p| p.name.clone()).collect();
        Template::render(
            "play",
            context! {
                code: room.code.clone(),
                players,
                question_placeholder: "Questions loading soon… (Step 2 will add 200 💕)"
            },
        )
    } else {
        Template::render(
            "play",
            context! {
                code,
                players: Vec::<String>::new(),
                question_placeholder: "Room not found."
            },
        )
    }
}

#[get("/result/<code>")]
fn result_get(code: String, state: &State<AppState>) -> Template {
    let map = state.rooms.read();
    if let Some(room) = map.get(&code) {
        // placeholder logic: equal split score just to render page
        let score = (room.players.len() as u32 * 42) % 100;
        let message = if score >= 85 {
            "Perfect Match 💍💖"
        } else if score >= 60 {
            "Good Match 💕"
        } else {
            "Nice Try 😅"
        };
        Template::render(
            "result",
            context! { code, score, message },
        )
    } else {
        Template::render(
            "result",
            context! { code, score: 0, message: "Room not found." },
        )
    }
          }
