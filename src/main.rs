mod config;
mod error;
mod pokemon;

use config::Config;
use error::Error;
use pokemon::*;

use clap::{arg, Args, Command, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use std::io;

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
    /// Generate shell completions
    Init(ShellName),
    /// Print list of all pokemon
    List,
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
struct ShellName {
    shell: Shell,
}

#[derive(Debug, Args)]
struct Name {
    /// Name of the pokemon to show
    name: String,

    /// Show an alternative form of the pokemon. Can be one of: mega, mega-x,
    /// mega-y, gmax, alola, hisui, galar, paldea
    #[clap(short, long, default_value = "regular")]
    form: String,

    /// Show the shiny version of the pokemon instead
    #[clap(short, long)]
    shiny: bool,

    /// Print pokedex entry (if it exists)
    #[clap(short, long)]
    info: bool,

    /// Do not display pokemon name
    #[clap(long)]
    no_title: bool,

    /// Left padding
    #[clap(long, default_value = "0")]
    padding_left: usize,
}

#[derive(Debug, Args)]
struct Random {
    /// Generation number, range (1-9), or list of generations (1,3,6)
    #[clap(default_value = "1-9")]
    generations: String,

    /// Print pokedex entry (if it exists)
    #[clap(short, long)]
    info: bool,

    /// Show the shiny version of the pokemon instead
    #[clap(short, long)]
    shiny: bool,

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

    /// Left padding
    #[clap(long, default_value = "0")]
    padding_left: usize,
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
            let slug = if name.form == "regular" {
                name.name.clone()
            } else {
                format!("{}-{}", name.name, name.form)
            };
            let art_path = if name.shiny {
                format!("colorscripts/shiny/{}", slug)
            } else {
                format!("colorscripts/regular/{}", slug)
            };
            let art = Asset::get(&art_path)
                .unwrap_or_else(|| panic!("Could not read pokemon art of '{}'", slug))
                .data;
            let art = str::from_utf8(&art).expect("Invalid UTF-8 in pokemon art");
            if !name.no_title {
                let pokemon_name = match pokemon.name.get(&config.language) {
                    Some(n) => n,
                    None => return Err(Error::InvalidLanguage(config.language.clone())),
                };
                print!("{: <1$}{pokemon_name}", "", name.padding_left);
                match name.form.as_str() {
                    "regular" => println!(),
                    other => println!(" ({other})"),
                }
            }
            if name.info {
                if let Some(description) = pokemon.desc.get(&config.language) {
                    description
                        .lines()
                        .for_each(|line| println!("{: <1$}{line}", "", name.padding_left))
                }
            }
            println!();
            art.lines()
                .for_each(|line| println!("{: <1$}{line}", "", name.padding_left))
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
    let pokemon = pokemon_db
        .iter()
        .filter(|p| start_gen <= p.gen && end_gen >= p.gen)
        .collect::<Vec<_>>();

    let pokemon = match pokemon.choose(&mut rand::thread_rng()) {
        Some(p) => Ok(p),
        None => Err(Error::InvalidGeneration(random.generations.clone())),
    }?;

    let mut forms = vec!["regular".to_string()];
    forms.extend_from_slice(&pokemon.forms);
    // Optional filters
    if random.no_mega {
        forms.retain(|f| !["mega", "mega-x", "mega-y"].contains(&f.as_str()));
    }
    if random.no_gmax {
        forms.retain(|f| f.as_str() != "gmax");
    }
    if random.no_regional {
        forms.retain(|f| !["alola", "galar", "hisui", "paldea"].contains(&f.as_str()));
    }
    // forms will never be empty, so safe to unwrap
    let form = forms.choose(&mut rand::thread_rng()).unwrap();
    let shiny = rand::thread_rng().gen_bool(config.shiny_rate) || random.shiny;

    show_pokemon_by_name(
        &Name {
            name: pokemon.slug.clone(),
            form: form.to_string(),
            shiny,
            info: random.info,
            no_title: random.no_title,
            padding_left: random.padding_left,
        },
        pokemon_db,
        config,
    )
}

fn build_cli() -> Command {
    let common_args = [
        arg!(-i --info "Print pokedex entry (if it exists)"),
        arg!(-s --shiny "Show the shiny pokemon version instead"),
        arg!(--"no-title" "Do not display pokemon name"),
        arg!(--"padding-left" "Set amount of padding to the left [default: 0]"),
    ];
    let completions = Command::new("completions")
        .about("Generate shell completions")
        .args([
            arg!(["bash"]),
            arg!(["zsh"]),
            arg!(["fish"]),
            arg!(["powershell"]),
            arg!(["elvish"]),
        ]);

    let list = Command::new("list").about("List all names of pokemons");
    let name = Command::new("name")
        .about("Select pokemon by name: eg. 'pikachu'")
        .arg(arg!([name] "Who's that pokemon!?"))
        .args(&common_args);

    let random = Command::new("random")
        .about("Show random pokemon")
        .arg(
            arg!([GENERATIONS] "Generation number, range (1-9), or list of generations (1,3,6) [default: 1-9]"),
        )
        .args(common_args)
        .args([
            arg!(--"no-mega" "Do not show mega pokemon"),
            arg!(--"no-gmax" "Do not show gigantamax pokemon"),
            arg!(--"no-regional" "Do not show regional pokemon"),
        ]);

    Command::new("krabby").subcommands([completions, list, name, random])
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn main() -> Result<(), Error> {
    let config = Config::load()?;
    let pokemon_db = Asset::get("pokemon.json").expect("Could not read pokemond db file");
    let pokemon = load_pokemon(&pokemon_db)?;
    let args = Cli::parse();
    match args.command {
        Commands::Init(shell) => print_completions(shell.shell, &mut build_cli()),
        Commands::List => list_pokemon_names(pokemon),
        Commands::Name(name) => show_pokemon_by_name(&name, pokemon, &config)?,
        Commands::Random(random) => show_random_pokemon(&random, pokemon, &config)?,
    }
    Ok(())
}
