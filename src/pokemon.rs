use serde::Deserialize;

use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;

use crate::config::Config;

#[derive(Debug, Deserialize)]
pub struct Pokemon {
    pub idx: u32,
    pub slug: String,
    pub gen: u8,
    pub form: String,
    pub name: HashMap<String, String>,
    pub desc: HashMap<String, String>,
}

pub fn load_pokemon_db(config: &Config) -> Result<Vec<Pokemon>, Box<dyn Error>> {
    let pokemon_db_path = format!("{}/pokemon.json", config.program_dir);
    let pokemon_db_path = Path::new(&pokemon_db_path);
    let pokemon_json_str = read_to_string(pokemon_db_path)?;
    let pokemon: Vec<Pokemon> = serde_json::from_str(&pokemon_json_str)?;
    Ok(pokemon)
}
