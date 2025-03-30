// main.rs

use std::str::FromStr;

use clap::{Parser, Subcommand};
use krabby::{
    Form, Generations, NameOptions, PokemonOptions, RandomOptions, error::Error, list_pokemon,
    pokemon_by_name, random_pokemon,
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Print list of all pokemon
    List {
        /// Generation number, range (1-9), or list of generations (1,3,6)
        #[clap(default_value = "1-9")]
        generations: String,
    },
    /// Select pokemon by name
    Name {
        /// Name of the pokemon to show
        name: String,
        /// Form (eg. regular, mega, etc.)
        #[clap(short, long, default_value = "regular")]
        form: String,
        #[clap(short, long)]
        info: bool,
        #[clap(short, long)]
        shiny: bool,
        #[clap(long)]
        no_title: bool,
        #[clap(long, default_value = "0")]
        padding_left: usize,
    },
    /// Show a random pokemon
    Random {
        /// Generation number, range (1-9), or list of generations (1,3,6)
        #[clap(default_value = "1-9")]
        generations: String,
        #[clap(long)]
        no_mega: bool,
        #[clap(long)]
        no_gmax: bool,
        #[clap(long)]
        no_regional: bool,
        #[clap(long)]
        no_variant: bool,
        #[clap(short, long)]
        info: bool,
        #[clap(short, long)]
        shiny: bool,
        #[clap(long)]
        no_title: bool,
        #[clap(long, default_value = "0")]
        padding_left: usize,
    },
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();

    let result = match args.command {
        Commands::List { generations } => {
            // Convert the generations string into the library’s Generations type.
            let gens = generations.parse::<Generations>()?;
            list_pokemon(&gens)?
        }
        Commands::Name {
            name,
            form,
            info,
            shiny,
            no_title,
            padding_left,
        } => {
            // Convert the form string to a Form.
            let form = Form::from_str(&form).unwrap();
            pokemon_by_name(&NameOptions {
                name,
                form,
                common: PokemonOptions {
                    info,
                    shiny,
                    no_title,
                    padding_left,
                },
            })?
        }
        Commands::Random {
            generations,
            no_mega,
            no_gmax,
            no_regional,
            no_variant,
            info,
            shiny,
            no_title,
            padding_left,
        } => {
            let gens = generations.parse::<Generations>()?;
            random_pokemon(&RandomOptions {
                generations: gens,
                no_mega,
                no_gmax,
                no_regional,
                no_variant,
                common: PokemonOptions {
                    info,
                    shiny,
                    no_title,
                    padding_left,
                },
            })?
        }
    };

    println!("{}", result);
    Ok(())
}
