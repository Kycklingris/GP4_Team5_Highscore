use actix_web::{
	get, http::header::ContentType, post, web, App, HttpResponse, HttpServer, Responder,
};

use rusqlite::Connection;

mod helper;
use helper::*;

mod highscore;
use highscore::*;

mod state;
use state::*;

#[get("/highscores/{level}")]
async fn get_highscores(
	level: web::Path<String>,
	_: actix_web::HttpRequest, data: web::Data<AppState>
) -> impl Responder {
	let highscores = data.get_highscores(level.to_string());

	let res_body = serde_json::to_string(&highscores).unwrap();

	HttpResponse::Ok()
		.content_type(ContentType::json())
		.body(res_body)
}

#[get("/top_ten/{level}")]
async fn get_top_ten(
	level: web::Path<String>,
	_: actix_web::HttpRequest,
	data: web::Data<AppState>,
) -> impl Responder {
	let highscores = data.get_highscores(level.to_string());
	let mut length = highscores.len();

	if length > 10 {
		length = 10;
	}
	let res_body = serde_json::to_string(&highscores[0..length]).unwrap();

	HttpResponse::Ok()
		.content_type(ContentType::json())
		.body(res_body)
}

#[post("/highscore")]
async fn set_highscore(
	req: web::Json<Highscore>, 
	data: web::Data<AppState>
) -> impl Responder {
	let mut rwlock = data.tmp.write().expect("lock is poisoned");
	let conn = Connection::open("./state/highscores.sqlite3").expect("Unable to read the database");

	let highscore = req.0.to_owned();

	let mut data: Vec<u8> = Vec::with_capacity(std::mem::size_of::<GhostLocation>() * highscore.ghost.len());
	for location in highscore.ghost {
		data.extend(unsafe { any_as_u8_slice(&location) });
	}

	conn.execute(
		"INSERT INTO highscores (level, name, score, time, ghost) VALUES (?1, ?2, ?3, ?4, ?5)",
		(&highscore.level, &highscore.name, &highscore.score, &highscore.time, &data),
	)
	.expect("Unable to add row to database");

	*rwlock = 1;
	HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let mut path = std::env::current_exe()?;
	path.pop();
	std::env::set_current_dir(path)?;

	let state = web::Data::new(AppState::load());

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::clone(&state))
			.service(get_highscores)
			.service(set_highscore)
			.service(get_top_ten)
	})
	.bind(("0.0.0.0", 80))?
	.bind(("0.0.0.0", 443))?
	// .bind(("127.0.0.1", 80))?
	// .bind(("127.0.0.1", 443))?
	.run()
	.await
}
