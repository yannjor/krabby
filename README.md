# krabby

Krabby is mostly a Rust rewrite of phoney badger's [pokemon-colorscripts](https://gitlab.com/phoneybadger/pokemon-colorscripts) with some extra features. It is around 7x faster than the original shell script ⚡.

![](https://i.imgur.com/MVzaS3k.png)

## Table of contents
* [Features](#features)
* [Installation](#installation)
  * [Arch Linux (and derivatives)](#arch-linux-and-derivatives)
  * [On other distros and MacOS](#on-other-distros-and-macos)
* [Usage](#usage)
  * [Examples](#examples)
* [Configuration](#configuration)
* [Similar projects](#similar-projects)

## Features
- Pokemon from every generation, including shinies, megas, gigantamax, and regional variants
- Print random pokemon (with filters for generations and different forms)
- Print pokemon by name
- Configuration file, right now only for language and shiny rate

## Installation

To build krabby, you will need Rust. Installation instructions can be found [here](https://www.rust-lang.org/learn/get-started).

### Arch Linux (and derivatives)

Download the PKGBUILD file from the repository, then run
```
makepkg -si
```

### On other distros and MacOS
Clone the respository, then run
```
sudo ./install.sh
```

## Usage
Run the help command `krabby help` to see the following help message.

```
USAGE:
    krabby <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help      Print this message or the help of the given subcommand(s)
    list      Print list of all pokemon
    name      Select pokemon by name. Generally spelled like in the games. A few exceptions are
                  nidoran-f, nidoran-m, mr-mime, farfetchd, flabebe type-null etc. Perhaps grep the
                  output of list if in doubt
    random    Show a random pokemon. This command can optionally be followed by a generation
                  number or range (1-8) to show random pokemon from a specific generation or range
                  of generations. The generations can be provided as a continuous range (eg. 1-3) or
                  as a list of generations (1,3,6)
```
To get more detailed information about a subcommand you can also view its help, for example
```
krabby help random
```
To get the help of the random subcommand.

### Examples
Print a specific pokemon
```
krabby name charizard
```
Print a specific shiny pokemon
```
krabby name spheal -s
```
Print a random pokemon (gens 1-8)
```
krabby random
```
Print random pokemon from generations 1-3
```
krabby random 1-3
```
Print a random pokemon from generations 1,3 and 6
```
krabby random 1,3,6
```
Print a random pokemon excluding megas, gigantamax and regional variants
```
krabby random --no-mega --no-gmax --no-regional
```

## Configuration

When the program is run, a TOML config file will automatically be created in the user's config
directory (usually `~/.config`) under `krabby/config.toml` if it doesn't exist already. 

```toml
# The language to use when printing the pokemon's name. Possible options include
# en (English), fr (French), de (German), ja (Japanese), ja_ro (Japanese with
# roman characters), zh_hans (Chinese with simplified characters), zh_hant (Chinese
# with traditional characters)
language = 'en'

# The probability to show a shiny pokemon when using the random command
shiny_rate = 0.0078125
```

## Similar projects
- [pokemon-colorscripts](https://gitlab.com/phoneybadger/pokemon-colorscripts)
- [pokeget](https://github.com/talwat/pokeget)
- [pokeshell](https://github.com/acxz/pokeshell)
