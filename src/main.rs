use clap::{Args, Parser, Subcommand};

use std::error::Error;
use std::fs::read_to_string;

const POKEART_REGULAR_DIR: &str = "./colorscripts/regular";
const POKEART_SHINY_DIR: &str = "./colorscripts/shiny";

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

    /// Do not display pokemon name
    #[clap(short, long)]
    title: bool,
}

#[derive(Debug, Args)]
struct Random {
    /// Generation number, range (1-8), or list of generations (1,3,6)
    generations: Option<String>,
}

fn list_pokemon_names() -> Result<(), Box<dyn Error>> {
    let list = read_to_string("./nameslist.txt")?;
    println!("{list}");
    Ok(())
}

fn show_pokemon_by_name(name: &Name) -> Result<(), Box<dyn Error>> {
    let art_path = if name.shiny {
        format!("{}/{}.txt", POKEART_SHINY_DIR, name.name)
    } else {
        format!("{}/{}.txt", POKEART_REGULAR_DIR, name.name)
    };
    let art = read_to_string(art_path)?;
    if name.title {
        println!("{}", name.name);
    }
    println!("{art}");
    Ok(())
}

fn show_random_pokemon(random: &Random) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    match args.command {
        Commands::List => list_pokemon_names()?,
        Commands::Name(name) => show_pokemon_by_name(&name)?,
        Commands::Random(random) => show_random_pokemon(&random)?,
    }
    Ok(())
}
