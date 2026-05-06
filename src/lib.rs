mod config;
pub mod error;
pub mod pokemon;

use config::Config;
use error::Error;
pub use pokemon::*;

use rust_embed::RustEmbed;
use std::str;

#[derive(Debug)]
pub struct PokemonOptions {
    pub info: bool,
    pub shiny: bool,
    pub no_title: bool,
    pub padding_left: usize,
}

#[derive(Debug)]
pub struct NameOptions {
    pub name: String,
    pub form: Form,
    pub common: PokemonOptions,
}

#[derive(Debug)]
pub struct RandomOptions {
    pub generations: Generations,
    pub no_mega: bool,
    pub no_gmax: bool,
    pub no_regional: bool,
    pub no_variant: bool,
    pub common: PokemonOptions,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

// Note: Here you may need to refactor your methods so that they return strings
// instead of directly printing to stdout.
pub fn list_pokemon(generations: &Generations) -> Result<String, Error> {
    let pokemon_db = load_db()?;
    Ok(pokemon_db.list_pokemon_names(generations))
}

pub fn pokemon_by_name(options: &NameOptions) -> Result<String, Error> {
    let pokemon_db = load_db()?;
    // For example, imagine we refactored show_pokemon_by_name to return a String.
    pokemon_db.show_pokemon_by_name(options)
}

pub fn random_pokemon(options: &RandomOptions) -> Result<String, Error> {
    let pokemon_db = load_db()?;
    pokemon_db.show_random_pokemon(options)
}

fn load_db() -> Result<PokemonDatabase, Error> {
    let config = Config::load().unwrap_or_default();
    let pokemon_db_file = Asset::get("pokemon.json").expect("Could not read pokemon db file");
    PokemonDatabase::load(&pokemon_db_file, config)
}
