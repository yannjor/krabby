mod config;
mod pokemon;

use config::Config;
use pokemon::*;

use clap::{Args, Parser, Subcommand};
use rand::seq::SliceRandom;
use rand::Rng;

use std::fs;
use std::path::Path;
use std::process;

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

    /// Do not display pokemon name
    #[clap(long)]
    no_title: bool,
}

#[derive(Debug, Args)]
struct Random {
    /// Generation number, range (1-8), or list of generations (1,3,6)
    #[clap(default_value = "1-8")]
    generations: String,

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

fn list_pokemon_names(pokemon_db: &[Pokemon]) {
    pokemon_db.iter().for_each(|p| println!("{}", p.slug));
}

fn show_pokemon_by_name(name: &Name, pokemon_db: &[Pokemon], config: &Config) {
    if let Some(pokemon) = pokemon_db.iter().find(|p| p.slug == name.name) {
        let art_path = if name.shiny {
            format!(
                "{}/colorscripts/shiny/{}.txt",
                config.program_dir, name.name
            )
        } else {
            format!(
                "{}/colorscripts/regular/{}.txt",
                config.program_dir, name.name
            )
        };
        let art_path = Path::new(&art_path);
        let art = fs::read_to_string(art_path)
            .unwrap_or_else(|_| panic!("Could not read pokemon art of '{}'", name.name));
        if !name.no_title {
            println!(
                "{}",
                pokemon
                    .name
                    .get(&config.language)
                    .unwrap_or_else(|| panic!("Invalid language '{}'", config.language))
            );
        }
        println!("{art}");
    } else {
        eprintln!("Invalid pokemon '{}'", name.name);
        process::exit(1);
    }
}

fn show_random_pokemon(random: &Random, pokemon_db: &[Pokemon], config: &Config) {
    let (start_gen, end_gen) = match random.generations.split_once('-') {
        Some(gens) => gens,
        None => {
            let gen_list = random.generations.split(',').collect::<Vec<_>>();
            let gen = gen_list.choose(&mut rand::thread_rng()).unwrap();
            (*gen, *gen)
        }
    };
    let start_gen = start_gen
        .parse::<u8>()
        .unwrap_or_else(|_| panic!("Failed to parse generation '{start_gen}'"));
    let end_gen = end_gen
        .parse::<u8>()
        .unwrap_or_else(|_| panic!("Failed to parse generation '{start_gen}'"));

    // Filter by generation
    let mut pokemon = pokemon_db
        .iter()
        .filter(|p| start_gen <= p.gen && end_gen >= p.gen)
        .collect::<Vec<_>>();

    // Optional filters
    if random.no_mega {
        pokemon = pokemon
            .iter()
            .copied()
            .filter(|p| !["mega", "mega-x", "mega-y"].contains(&p.form.as_str()))
            .collect::<Vec<_>>();
    }
    if random.no_gmax {
        pokemon = pokemon
            .iter()
            .copied()
            .filter(|p| p.form != "gmax")
            .collect::<Vec<_>>();
    }
    if random.no_regional {
        pokemon = pokemon
            .iter()
            .copied()
            .filter(|p| !["alola", "galar", "hisui"].contains(&p.form.as_str()))
            .collect::<Vec<_>>();
    }

    let pokemon = pokemon
        .choose(&mut rand::thread_rng())
        .expect("Generation must be between 1 and 8");
    let shiny = rand::thread_rng().gen_bool(config.shiny_rate);
    show_pokemon_by_name(
        &Name {
            name: pokemon.slug.clone(),
            shiny,
            no_title: random.no_title,
        },
        pokemon_db,
        config,
    );
}

fn main() {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {e}");
            std::process::exit(1)
        }
    };
    let pokemon = match load_pokemon_db(&config) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load pokemon db: {e}");
            std::process::exit(1)
        }
    };
    let args = Cli::parse();
    match args.command {
        Commands::List => list_pokemon_names(&pokemon),
        Commands::Name(name) => show_pokemon_by_name(&name, &pokemon, &config),
        Commands::Random(random) => show_random_pokemon(&random, &pokemon, &config),
    }
}
