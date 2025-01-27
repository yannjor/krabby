mod config;
mod error;
mod pokemon;

use config::Config;
use error::Error;
use pokemon::*;

use clap::{Args, Parser, Subcommand};
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
    List(List),
    /// Select pokemon by name. Generally spelled like in the games.
    /// A few exceptions are nidoran-f, nidoran-m, mr-mime, farfetchd,
    /// flabebe type-null etc. Perhaps grep the output of list if in doubt.
    Name(Name),
    /// Show a random pokemon. This command can optionally be followed by a
    /// generation number or range (1-9) to show random pokemon from a specific
    /// generation or range of generations. The generations can be provided as
    /// a continuous range (eg. 1-3) or as a list of generations (1,3,6)
    Random(Random),
}

#[derive(Debug, Args)]
struct CommonArgs {
    /// Print pokedex entry (if it exists)
    #[clap(short, long)]
    info: bool,

    /// Show the shiny version of the pokemon instead
    #[clap(short, long)]
    shiny: bool,

    /// Do not display pokemon name
    #[clap(long)]
    no_title: bool,

    /// Left padding
    #[clap(long, default_value = "0")]
    padding_left: usize,
}

#[derive(Debug, Args)]
struct List {
    /// Generation number, range (1-9), or list of generations (1,3,6)
    #[clap(default_value = "1-9")]
    generations: Generations,
}

#[derive(Debug, Args)]
struct Name {
    /// Name of the pokemon to show
    name: String,

    /// Show an alternative form of the pokemon. For example: mega, mega-x,
    /// mega-y, gmax, alola, hisui, galar, paldea
    #[clap(short, long, default_value = "regular", value_parser = Form::from_str)]
    form: Form,

    #[clap(flatten)]
    common: CommonArgs,
}

#[derive(Debug, Args)]
struct Random {
    /// Generation number, range (1-9), or list of generations (1,3,6)
    #[clap(default_value = "1-9")]
    generations: Generations,

    /// Do not show mega pokemon
    #[clap(long)]
    no_mega: bool,

    /// Do not show gigantamax pokemon
    #[clap(long)]
    no_gmax: bool,

    /// Do not show regional pokemon
    #[clap(long)]
    no_regional: bool,

    /// Do not show any variant-form pokemon
    #[clap(long)]
    no_variant: bool,

    #[clap(flatten)]
    common: CommonArgs,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

fn main() -> Result<(), Error> {
    let config = Config::load()?;
    let pokemon_db_file = Asset::get("pokemon.json").expect("Could not read pokemon db file");
    let pokemon_db = PokemonDatabase::load(&pokemon_db_file, config)?;
    let args = Cli::parse();
    match args.command {
        Commands::List(list_args) => pokemon_db.list_pokemon_names(list_args.generations),
        Commands::Name(name) => pokemon_db.show_pokemon_by_name(&name)?,
        Commands::Random(random) => pokemon_db.show_random_pokemon(&random)?,
    }
    Ok(())
}
