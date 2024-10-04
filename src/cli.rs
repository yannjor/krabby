use clap::{arg, Command};
use clap_complete::{generate, Generator};
use std::io;

pub fn build() -> Command {
    let common_args = [
        arg!(-i --info "Print pokedex entry (if it exists)"),
        arg!(-s --shiny "Show the shiny pokemon version instead"),
        arg!(--"no-title" "Do not display pokemon name"),
        arg!(--"padding-left" "Set amount of padding to the left [default: 0]"),
    ];
    let init = Command::new("init")
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

    Command::new("krabby").subcommands([init, list, name, random])
}

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
