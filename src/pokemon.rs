use serde::Deserialize;

use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;

const POKEMON_DB_PATH: &str = "./pokemon.json";

#[derive(Debug, Deserialize)]
pub struct Pokemon {
    pub idx: u32,
    pub slug: String,
    pub gen: u8,
    pub form: String,
    pub name: HashMap<String, String>,
}

pub fn load_pokemon_db() -> Result<Vec<Pokemon>, Box<dyn Error>> {
    let pokemon_json_str = read_to_string(POKEMON_DB_PATH)?;
    let pokemon: Vec<Pokemon> = serde_json::from_str(&pokemon_json_str)?;
    Ok(pokemon)
}
