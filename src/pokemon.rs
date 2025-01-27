use rand::seq::IteratorRandom;
use rand::Rng;
use rust_embed::EmbeddedFile;
use serde::Deserialize;

use std::collections::HashMap;
use std::iter::once;
use std::str::{self, FromStr};

use crate::config::Config;
use crate::error::Error;
use crate::{Asset, CommonArgs, Name, Random};

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Form {
    Regular,
    Mega,
    MegaX,
    MegaY,
    Gmax,
    Alola,
    Galar,
    Hisui,
    Paldea,
    #[serde(untagged)]
    Other(String), // Catch-all for some one-offs like "primal"
}

impl Form {
    pub fn from_str(form: &str) -> Result<Form, std::convert::Infallible> {
        Ok(match form {
            "regular" => Form::Regular,
            "mega" => Form::Mega,
            "mega-x" => Form::MegaX,
            "mega-y" => Form::MegaY,
            "gmax" => Form::Gmax,
            "alola" => Form::Alola,
            "galar" => Form::Galar,
            "hisui" => Form::Hisui,
            "paldea" => Form::Paldea,
            _ => Form::Other(form.to_string()),
        })
    }

    pub fn as_str(&self) -> &str {
        match self {
            Form::Regular => "regular",
            Form::Mega => "mega",
            Form::MegaX => "mega-x",
            Form::MegaY => "mega-y",
            Form::Gmax => "gmax",
            Form::Alola => "alola",
            Form::Galar => "galar",
            Form::Hisui => "hisui",
            Form::Paldea => "paldea",
            Form::Other(s) => s,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Pokemon {
    /// The asset slug of the pokemon, before any form/shiny resolution
    pub slug: String,
    /// The generation of the pokemon
    pub gen: u8,
    /// The name of the pokemon in different languages
    pub name: HashMap<String, String>,
    /// The description of the pokemon in different languages
    pub desc: HashMap<String, String>,
    /// The different forms of the pokemon
    pub forms: Vec<Form>,
}

impl Pokemon {
    /// Get all forms of the pokemon except those in the exclude vector
    pub fn get_filtered_forms(&self, exclude: &[Form]) -> Vec<Form> {
        self.forms
            .iter()
            .filter(|f| {
                !exclude.iter().any(|ex| match (ex, f) {
                    (Form::Other(_), Form::Other(_)) => true,
                    _ => ex == *f,
                })
            })
            .cloned()
            .collect()
    }

    /// Get the asset slug for the pokemon and resolve shininess
    pub fn get_art_path(&self, form: &Form, shiny: bool) -> Result<String, Error> {
        Ok(format!(
            "colorscripts/{}/{}",
            if shiny { "shiny" } else { "regular" },
            self.get_form_slug(form)?
        ))
    }

    /// Get the asset slug for the pokemon form
    pub fn get_form_slug(&self, form: &Form) -> Result<String, Error> {
        if form == &Form::Regular {
            Ok(self.slug.clone())
        } else if self.forms.contains(form) {
            Ok(format!("{}-{}", self.slug, form.as_str()))
        } else {
            Err(Error::InvalidPokemonForm(
                self.slug.clone(),
                form.as_str().to_string(),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Generations(Vec<u8>);

impl FromStr for Generations {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let gens: Vec<u8> = if let Some((start_gen, end_gen)) = input.split_once('-') {
            let start_gen = start_gen.parse::<u8>();
            let end_gen = end_gen.parse::<u8>();
            match (start_gen, end_gen) {
                (Ok(s), Ok(e)) => (s..=e).collect(),
                _ => return Err(Error::InvalidGeneration(input.to_string())),
            }
        } else {
            input
                .split(',')
                .map(|gen| gen.parse::<u8>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| Error::InvalidGeneration(input.to_string()))?
        };

        if gens.is_empty() || gens.iter().any(|&g| !(1..=9).contains(&g)) {
            return Err(Error::InvalidGeneration(input.to_string()));
        }

        Ok(Generations(gens))
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
    pub fn filter_by_generation<'a>(
        &'a self,
        generations: &'a Generations,
    ) -> impl Iterator<Item = &'a Pokemon> {
        self.pokemon
            .iter()
            .filter(|p| generations.0.contains(&p.gen))
    }

    /// Returns a vector of all pokemon
    pub fn get_all(&self) -> &Vec<Pokemon> {
        &self.pokemon
    }

    /// Prints the names of all pokemon from the given generations into stdout
    pub fn list_pokemon_names(&self, generations: Generations) {
        self.filter_by_generation(&generations)
            .for_each(|p| println!("{}", p.slug))
    }

    /// Prints the pokemon with the given name into stdout
    pub fn show_pokemon_by_name(&self, name: &Name) -> Result<(), Error> {
        let pokemon = self
            .get_all()
            .iter()
            .find(|p| p.slug == name.name)
            .ok_or_else(|| Error::InvalidPokemon(name.name.clone()))?;

        if name.form != Form::Regular && !pokemon.forms.contains(&name.form) {
            return Err(Error::InvalidPokemonForm(
                name.name.clone(),
                name.form.as_str().to_string(),
            ));
        }

        let art_path = pokemon.get_art_path(&name.form, name.common.shiny)?;

        let art = Asset::get(&art_path)
            .unwrap_or_else(|| panic!("Could not read pokemon art from '{}'", art_path))
            .data;
        let art = str::from_utf8(&art).expect("Invalid UTF-8 in pokemon art");

        let padding_left = " ".repeat(name.common.padding_left);

        if !name.common.no_title {
            if let Some(pokemon_name) = pokemon.name.get(&self.config.language) {
                print!("{padding_left}{pokemon_name}");
                if name.form != Form::Regular {
                    print!(" ({})", name.form.as_str());
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
                    .for_each(|line| println!("{padding_left}{line}"))
            }
        }
        println!();
        art.lines()
            .for_each(|line| println!("{padding_left}{line}"));

        Ok(())
    }

    /// Prints a random pokemon into stdout
    pub fn show_random_pokemon(&self, random: &Random) -> Result<(), Error> {
        let pokemon = self
            .filter_by_generation(&random.generations)
            .choose(&mut rand::thread_rng())
            .unwrap();

        let mut exclude_forms = Vec::new();
        if random.no_mega || random.no_variant {
            exclude_forms.extend_from_slice(&[Form::Mega, Form::MegaX, Form::MegaY]);
        }
        if random.no_gmax || random.no_variant {
            exclude_forms.push(Form::Gmax);
        }
        if random.no_regional || random.no_variant {
            exclude_forms.extend_from_slice(&[Form::Alola, Form::Galar, Form::Hisui, Form::Paldea]);
        }
        if random.no_variant {
            exclude_forms.push(Form::Other(String::new()));
        }

        let forms = pokemon.get_filtered_forms(&exclude_forms);

        let form = forms
            .iter()
            .chain(once(&Form::Regular))
            .choose(&mut rand::thread_rng())
            .unwrap_or_else(|| panic!("No forms available for {}, somehow", pokemon.slug))
            .to_owned();

        let shiny = rand::thread_rng().gen_bool(self.config.shiny_rate) || random.common.shiny;

        self.show_pokemon_by_name(&Name {
            name: pokemon.slug.clone(),
            form,
            common: CommonArgs {
                shiny,
                info: random.common.info,
                no_title: random.common.no_title,
                padding_left: random.common.padding_left,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_embed::RustEmbed;
    use std::collections::HashMap;

    #[derive(RustEmbed)]
    #[folder = "assets/"]
    struct TestAssets;

    fn create_test_pokemon() -> Pokemon {
        Pokemon {
            slug: "test".to_string(),
            gen: 1,
            name: HashMap::new(),
            desc: HashMap::new(),
            forms: vec![Form::Regular, Form::Mega, Form::Gmax],
        }
    }

    #[test]
    fn test_pokemon_get_filtered_forms() {
        let pokemon = create_test_pokemon();

        let exclude = vec![Form::Mega];
        let filtered_forms = pokemon.get_filtered_forms(&exclude);
        assert_eq!(filtered_forms, vec![Form::Regular, Form::Gmax]);

        let exclude = vec![Form::Other("unknown".to_string())];
        let filtered_forms = pokemon.get_filtered_forms(&exclude);
        assert_eq!(filtered_forms, pokemon.forms);
    }

    #[test]
    fn test_pokemon_get_form_slug() {
        let pokemon = create_test_pokemon();

        assert_eq!(pokemon.get_form_slug(&Form::Regular).unwrap(), "test");
        assert_eq!(pokemon.get_form_slug(&Form::Mega).unwrap(), "test-mega");
        assert!(pokemon
            .get_form_slug(&Form::Other("nonexistant".to_string()))
            .is_err());
    }

    #[test]
    fn test_generations_from_str() {
        assert_eq!(Generations::from_str("2").unwrap().0, vec![2]);
        assert_eq!(Generations::from_str("1-3").unwrap().0, vec![1, 2, 3]);
        assert_eq!(Generations::from_str("1,3,5").unwrap().0, vec![1, 3, 5]);
        assert!(Generations::from_str("0").is_err());
        assert!(Generations::from_str("256").is_err());
        assert!(Generations::from_str("1-100").is_err());
        assert!(Generations::from_str("a,b,c").is_err());
    }

    #[test]
    fn test_pokemon_database_load() {
        let config = Config {
            language: "en".to_string(),
            shiny_rate: 0.0,
        };
        let pokemon_db = TestAssets::get("pokemon.json").expect("pokemon.json not found");
        let db =
            PokemonDatabase::load(&pokemon_db, config).expect("Failed to load Pokemon database");

        assert!(
            !db.get_all().is_empty(),
            "Pokemon database should not be empty"
        );
    }

    #[test]
    fn test_pokemon_database_filter_by_generation() {
        let config = Config {
            language: "en".to_string(),
            shiny_rate: 0.1,
        };
        let pokemon_db = TestAssets::get("pokemon.json").expect("pokemon.json not found");
        let db =
            PokemonDatabase::load(&pokemon_db, config).expect("Failed to load Pokemon database");

        let generations = Generations::from_str("1-3").unwrap();
        let filtered_pokemon: Vec<&Pokemon> = db.filter_by_generation(&generations).collect();
        assert!(
            !filtered_pokemon.is_empty(),
            "Filtered pokemon should not be empty"
        );
    }

    #[test]
    fn test_pokemon_database_show_pokemon_by_name() {
        let config = Config {
            language: "en".to_string(),
            shiny_rate: 0.0,
        };
        let pokemon_db = TestAssets::get("pokemon.json").expect("pokemon.json not found");
        let db =
            PokemonDatabase::load(&pokemon_db, config).expect("Failed to load Pokemon database");

        let name = Name {
            name: "mewtwo".to_string(),
            form: Form::Regular,
            common: CommonArgs {
                shiny: false,
                info: false,
                no_title: false,
                padding_left: 0,
            },
        };

        assert!(db.show_pokemon_by_name(&name).is_ok());
    }

    #[test]
    fn test_pokemon_database_show_random_pokemon() {
        let config = Config {
            language: "en".to_string(),
            shiny_rate: 0.0,
        };
        let pokemon_db = TestAssets::get("pokemon.json").expect("pokemon.json not found");
        let db =
            PokemonDatabase::load(&pokemon_db, config).expect("Failed to load Pokemon database");

        let random = Random {
            generations: Generations::from_str("1-3").unwrap(),
            no_mega: false,
            no_gmax: false,
            no_regional: false,
            no_variant: false,
            common: CommonArgs {
                shiny: false,
                info: false,
                no_title: false,
                padding_left: 0,
            },
        };

        assert!(db.show_random_pokemon(&random).is_ok());
    }
}
