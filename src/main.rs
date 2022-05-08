mod config;
mod error;
mod pokemon;

use config::Config;
use error::Error;
use pokemon::*;

use clap::{Args, Parser, Subcommand};
use rand::seq::SliceRandom;
use rand::Rng;
use rust_embed::RustEmbed;

use std::str;

/// Print pokemon sprites in your terminal
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Print list of all pokemon
    List,
    /// Select pokemon by name. Generally spelled like in the games.
    /// A few exceptions are nidoran-f, nidoran-m, mr-mime, farfetchd,
    /// flabebe type-null etc. Perhaps grep the output of list if in doubt.
    Name(Name),
    /// Show a random pokemon. This command can optionally be followed by a
    /// generation number or range (1-8) to show random pokemon from a specific
    /// generation or range of generations. The generations can be provided as
    /// a continuous range (eg. 1-3) or as a list of generations (1,3,6)
    Random(Random),
}

#[derive(Debug, Args)]
struct Name {
    /// Name of the pokemon to show
    name: String,

    /// Show the shiny version of the pokemon instead
    #[clap(short, long)]
    shiny: bool,

    /// Print pokedex entry (if it exists)
    #[clap(short, long)]
    info: bool,

    /// Do not display pokemon name
    #[clap(long)]
    no_title: bool,
}

#[derive(Debug, Args)]
struct Random {
    /// Generation number, range (1-8), or list of generations (1,3,6)
    #[clap(default_value = "1-8")]
    generations: String,

    /// Print pokedex entry (if it exists)
    #[clap(short, long)]
    info: bool,

    /// Do not display pokemon name
    #[clap(long)]
    no_title: bool,

    /// Do not show mega pokemon
    #[clap(long)]
    no_mega: bool,

    /// Do not show gigantamax pokemon
    #[clap(long)]
    no_gmax: bool,

    /// Do not show regional pokemon
    #[clap(long)]
    no_regional: bool,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

fn list_pokemon_names(pokemon_db: Vec<Pokemon>) {
    pokemon_db.iter().for_each(|p| println!("{}", p.slug));
}

fn show_pokemon_by_name(
    name: &Name,
    pokemon_db: Vec<Pokemon>,
    config: &Config,
) -> Result<(), Error> {
    match pokemon_db.iter().find(|p| p.slug == name.name) {
        Some(pokemon) => {
            let art_path = if name.shiny {
                format!("colorscripts/shiny/{}", name.name)
            } else {
                format!("colorscripts/regular/{}", name.name)
            };
            let art = Asset::get(&art_path)
                .unwrap_or_else(|| panic!("Could not read pokemon art of '{}'", name.name))
                .data;
            let art = str::from_utf8(&art).expect("Invalid UTF-8 in pokemon art");
            if !name.no_title {
                let name = match pokemon.name.get(&config.language) {
                    Some(n) => n,
                    None => return Err(Error::InvalidLanguage(config.language.clone())),
                };
                print!("{name}");
                match pokemon.form.as_str() {
                    "normal" => println!(),
                    other => println!(" ({other})"),
                }
            }
            if name.info {
                match pokemon.desc.get(&config.language) {
                    Some(d) => println!("{d}"),
                    None => (),
                }
            }
            println!("\n{art}");
        }
        None => {
            return Err(Error::InvalidPokemon(name.name.clone()));
        }
    }
    Ok(())
}

fn show_random_pokemon(
    random: &Random,
    pokemon_db: Vec<Pokemon>,
    config: &Config,
) -> Result<(), Error> {
    let (start_gen, end_gen) = match random.generations.split_once('-') {
        Some(gens) => gens,
        None => {
            let gen_list = random.generations.split(',').collect::<Vec<_>>();
            let gen = gen_list.choose(&mut rand::thread_rng()).unwrap();
            (*gen, *gen)
        }
    };

    let start_gen = start_gen.parse::<u8>();
    let end_gen = end_gen.parse::<u8>();
    let (start_gen, end_gen) = match (start_gen, end_gen) {
        (Ok(s), Ok(e)) => (s, e),
        _ => return Err(Error::InvalidGeneration(random.generations.clone())),
    };

    // Filter by generation
    let mut pokemon = pokemon_db
        .iter()
        .filter(|p| start_gen <= p.gen && end_gen >= p.gen)
        .collect::<Vec<_>>();

    // Optional filters
    if random.no_mega {
        pokemon.retain(|p| !["mega", "mega-x", "mega-y"].contains(&p.form.as_str()));
    }
    if random.no_gmax {
        pokemon.retain(|p| p.form != "gmax");
    }
    if random.no_regional {
        pokemon.retain(|p| !["alola", "galar", "hisui"].contains(&p.form.as_str()));
    }

    let pokemon = match pokemon.choose(&mut rand::thread_rng()) {
        Some(p) => Ok(p),
        None => Err(Error::InvalidGeneration(random.generations.clone())),
    }?;

    let shiny = rand::thread_rng().gen_bool(config.shiny_rate);
    show_pokemon_by_name(
        &Name {
            name: pokemon.slug.clone(),
            shiny,
            info: random.info,
            no_title: random.no_title,
        },
        pokemon_db,
        config,
    )
}

fn main() -> Result<(), Error> {
    let config = Config::load()?;
    let pokemon_db = Asset::get("pokemon.json").expect("Could not read pokemond db file");
    let pokemon = load_pokemon(&pokemon_db)?;
    let args = Cli::parse();
    match args.command {
        Commands::List => list_pokemon_names(pokemon),
        Commands::Name(name) => show_pokemon_by_name(&name, pokemon, &config)?,
        Commands::Random(random) => show_random_pokemon(&random, pokemon, &config)?,
    }
    Ok(())
}
