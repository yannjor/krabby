mod pokemon;

use pokemon::*;

use clap::{Args, Parser, Subcommand};
use rand::seq::SliceRandom;
use rand::Rng;

use std::error::Error;
use std::fs::read_to_string;

const POKEART_REGULAR_DIR: &str = "./colorscripts/regular";
const POKEART_SHINY_DIR: &str = "./colorscripts/shiny";
const SHINY_RATE: f64 = 1.0 / 128.0;
const LANG: &str = "en";

/// CLI utility to print out unicode image of a pokemon in your shell
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

    /// Display pokemon name
    #[clap(short, long)]
    title: bool,
}

#[derive(Debug, Args)]
struct Random {
    /// Generation number, range (1-8), or list of generations (1,3,6)
    #[clap(default_value = "1-8")]
    generations: String,

    /// Display pokemon name
    #[clap(short, long)]
    title: bool,

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

fn show_pokemon_by_name(name: &Name, pokemon_db: &[Pokemon]) -> Result<(), Box<dyn Error>> {
    if let Some(pokemon) = pokemon_db.iter().find(|p| p.slug == name.name) {
        let art_path = if name.shiny {
            format!("{}/{}.txt", POKEART_SHINY_DIR, name.name)
        } else {
            format!("{}/{}.txt", POKEART_REGULAR_DIR, name.name)
        };
        let art = read_to_string(art_path)?;
        if name.title {
            println!(
                "{}",
                pokemon
                    .name
                    .get(LANG)
                    .unwrap_or_else(|| panic!("Invalid language '{LANG}'"))
            );
        }
        println!("{art}");
    } else {
        eprintln!("Invalid pokemon '{}'", name.name);
    }
    Ok(())
}

fn show_random_pokemon(random: &Random, pokemon_db: &[Pokemon]) -> Result<(), Box<dyn Error>> {
    let (start_gen, end_gen) = match random.generations.split_once('-') {
        Some(gens) => gens,
        None => {
            let gen_list = random.generations.split(',').collect::<Vec<_>>();
            let gen = gen_list
                .choose(&mut rand::thread_rng())
                .expect("Invalid generation");
            (*gen, *gen)
        }
    };
    let start_gen = start_gen.parse::<u8>()?;
    let end_gen = end_gen.parse::<u8>()?;
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

    let pokemon = pokemon.choose(&mut rand::thread_rng()).unwrap();
    let shiny = rand::thread_rng().gen_bool(SHINY_RATE);
    show_pokemon_by_name(
        &Name {
            name: pokemon.slug.clone(),
            shiny,
            title: random.title,
        },
        pokemon_db,
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let pokemon = load_pokemon_db()?;
    let args = Cli::parse();
    match args.command {
        Commands::List => list_pokemon_names(&pokemon),
        Commands::Name(name) => show_pokemon_by_name(&name, &pokemon)?,
        Commands::Random(random) => show_random_pokemon(&random, &pokemon)?,
    }
    Ok(())
}
