use rand::seq::SliceRandom;
use rand::Rng;
use rust_embed::EmbeddedFile;
use serde::Deserialize;

use std::collections::HashMap;
use std::str;

use crate::config::Config;
use crate::error::Error;
use crate::{Asset, CommonArgs, Name, Random};

#[derive(Debug, Deserialize)]
pub struct Pokemon {
    pub slug: String,
    pub gen: u8,
    pub name: HashMap<String, String>,
    pub desc: HashMap<String, String>,
    pub forms: Vec<String>,
}

impl Pokemon {
    /// Get all forms of the pokemon except those in the exclude vector
    pub fn get_filtered_forms(&self, exclude: &[&str]) -> Vec<String> {
        self.forms
            .iter()
            .filter(|f| !exclude.contains(&f.as_str()))
            .cloned()
            .collect()
    }
}

trait GenerationParser {
    fn parse_generations(&self) -> Result<(u8, u8), Error>;
}

impl GenerationParser for &str {
    fn parse_generations(&self) -> Result<(u8, u8), Error> {
        let (start_gen, end_gen) = match self.split_once('-') {
            Some(gens) => gens,
            None => {
                let gen_list = self.split(',').collect::<Vec<_>>();
                let gen = gen_list.choose(&mut rand::thread_rng()).unwrap();
                (*gen, *gen)
            }
        };

        let start_gen = start_gen.parse::<u8>();
        let end_gen = end_gen.parse::<u8>();
        match (start_gen, end_gen) {
            (Ok(s), Ok(e)) => Ok((s, e)),
            _ => Err(Error::InvalidGeneration(self.to_string())),
        }
    }
}

pub struct PokemonDatabase {
    pokemon: Vec<Pokemon>,
    config: Config,
}

impl PokemonDatabase {
    pub fn load(pokemon_db: &EmbeddedFile, config: Config) -> Result<Self, Error> {
        let pokemon_json_str =
            str::from_utf8(&pokemon_db.data).expect("Invalid UTF-8 in pokemon db");
        let pokemon: Vec<Pokemon> = serde_json::from_str(pokemon_json_str)?;
        Ok(Self { pokemon, config })
    }

    /// Filter pokemon by generation
    pub fn filter_by_generation(&self, generations: &str) -> Result<Vec<&Pokemon>, Error> {
        let (start_gen, end_gen) = generations.parse_generations()?;
        Ok(self
            .pokemon
            .iter()
            .filter(|p| start_gen <= p.gen && p.gen <= end_gen)
            .collect())
    }

    /// Returns a vector of all pokemon
    pub fn get_all(&self) -> &Vec<Pokemon> {
        &self.pokemon
    }

    /// Prints the names of all pokemon from the given generations into stdout
    pub fn list_pokemon_names(&self, generations: &str) -> Result<(), Error> {
        let filtered_pokemon = self.filter_by_generation(generations)?;
        filtered_pokemon.iter().for_each(|p| println!("{}", p.slug));
        Ok(())
    }

    /// Prints the pokemon with the given name into stdout
    pub fn show_pokemon_by_name(&self, name: &Name) -> Result<(), Error> {
        let pokemon = self
            .get_all()
            .iter()
            .find(|p| p.slug == name.name)
            .ok_or_else(|| Error::InvalidPokemon(name.name.clone()))?;

        let slug = if name.form == "regular" {
            name.name.clone()
        } else {
            format!("{}-{}", name.name, name.form)
        };

        let art_path = format!(
            "colorscripts/{}/{}",
            if name.common.shiny {
                "shiny"
            } else {
                "regular"
            },
            slug
        );

        let art = Asset::get(&art_path)
            .unwrap_or_else(|| panic!("Could not read pokemon art of '{}'", slug))
            .data;
        let art = str::from_utf8(&art).expect("Invalid UTF-8 in pokemon art");

        if !name.common.no_title {
            if let Some(pokemon_name) = pokemon.name.get(&self.config.language) {
                print!("{: <1$}{pokemon_name}", "", name.common.padding_left);
                if name.form != "regular" {
                    print!(" ({})", name.form);
                }
                println!();
            } else {
                return Err(Error::InvalidLanguage(self.config.language.clone()));
            }
        }

        if name.common.info {
            if let Some(description) = pokemon.desc.get(&self.config.language) {
                description
                    .lines()
                    .for_each(|line| println!("{: <1$}{line}", "", name.common.padding_left))
            }
        }
        println!();
        art.lines()
            .for_each(|line| println!("{: <1$}{line}", "", name.common.padding_left));

        Ok(())
    }

    /// Prints a random pokemon into stdout
    pub fn show_random_pokemon(&self, random: &Random) -> Result<(), Error> {
        let filtered_pokemon = self.filter_by_generation(&random.generations)?;

        let pokemon = match filtered_pokemon.choose(&mut rand::thread_rng()) {
            Some(p) => Ok(p),
            None => Err(Error::InvalidGeneration(random.generations.clone())),
        }?;

        let mut exclude_forms = Vec::new();
        if random.no_mega {
            exclude_forms.extend_from_slice(&["mega", "mega-x", "mega-y"]);
        }
        if random.no_gmax {
            exclude_forms.push("gmax");
        }
        if random.no_regional {
            exclude_forms.extend_from_slice(&["alola", "galar", "hisui", "paldea"]);
        }

        let forms = pokemon.get_filtered_forms(&exclude_forms);

        let form = forms
            .choose(&mut rand::thread_rng())
            .map(|s| s.as_str())
            .unwrap_or("regular");

        let shiny = rand::thread_rng().gen_bool(self.config.shiny_rate) || random.common.shiny;

        self.show_pokemon_by_name(&Name {
            name: pokemon.slug.clone(),
            form: form.to_string(),
            common: CommonArgs {
                shiny,
                info: random.common.info,
                no_title: random.common.no_title,
                padding_left: random.common.padding_left,
            },
        })
    }
}
