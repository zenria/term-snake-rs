use rustbreak::deser::Yaml;
use rustbreak::FileDatabase;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Borrow;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Highscore {
    name: String,
    score: u32,
}

impl PartialEq for Highscore {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl PartialOrd for Highscore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Highscores {
    scores: Vec<Highscore>,
}

impl Default for Highscores {
    fn default() -> Self {
        Self { scores: Vec::new() }
    }
}

const PATH: &'static str = "~/.snake_highscore.yml";

pub struct HighscoresStore {
    db: FileDatabase<Highscores, Yaml>,
}
impl HighscoresStore {
    pub fn new() -> Self {
        let path = shellexpand::tilde(PATH).to_owned();
        let db = FileDatabase::from_path(path.into_owned(), Default::default()).unwrap();
        Self { db }
    }
}
