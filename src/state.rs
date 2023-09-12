use std::sync::RwLock;


use rusqlite::Connection;

use crate::highscore::*;


use crate::helper::*;

pub struct AppState {
	pub tmp: RwLock<u32>,
}

impl AppState {
	pub fn load() -> Self {
		let mut path = std::env::current_exe().expect("Unable to get the current exe path");
		path.pop();
		path.push("state");

		std::fs::create_dir_all(path).expect("Unable to create the directory");

		let conn =
			Connection::open("./state/highscores.sqlite3").expect("Unable to open the database");

		conn.execute(
			"CREATE TABLE if not exists highscores (
				level   TEXT NOT NULL,
				name    TEXT NOT NULL,
				score   INTEGER NOT NULL,
				time    FLOAT NOT NULL,
				ghost   LONGBLOB NOT NULL
			)",
			(),
		)
		.expect("unable to create db table");

		Self {
			tmp: RwLock::new(1),
		}
	}

	pub fn get_highscores(&self, search_level: String) -> Highscores {
		let tmp = self.tmp.read().expect("unable to lock the rwlock");
		let mut highscores: Vec<Highscore> = Vec::new();
		let conn =
			Connection::open("./state/highscores.sqlite3").expect("Unable to read the database");
		let mut stmt = conn
			.prepare("SELECT level, name, score, time, ghost FROM highscores WHERE level=:level ORDER BY score DESC")
			.expect("Unable to read the database");
		let highscores_iter = stmt
			.query_map(&[(":level", search_level.as_str())], |row| {			
				let ghost_data: Vec<u8> = row.get(4)?;
				let mut ghost: Vec<GhostLocation> = Vec::with_capacity(ghost_data.len() / std::mem::size_of::<GhostLocation>());

				for i in (0..ghost_data.len()).step_by(std::mem::size_of::<GhostLocation>()) {
					ghost.push(unsafe { any_from_u8_slice::<GhostLocation>(&ghost_data[i..(i + std::mem::size_of::<GhostLocation>())]) });
				}

				let highscore = Highscore {
					level: row.get(0)?,
					name: row.get(1)?,
					score: row.get(2)?,
					time: row.get(3)?,
					ghost,
				};
				Ok(highscore)
			})
			.expect("Unable to map the database");

		for row in highscores_iter {
			match row {
				Ok(r) => highscores.push(r),
				Err(_) => {}
			}
		}
		if *tmp == 1 {
			drop(tmp)
		}

		highscores.into()
	}
}
