use thiserror::Error;

#[derive(Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Failed to load pokemon db: {0}")]
    PokemonDb(#[from] serde_json::Error),

    #[error("Invalid pokemon `{0}`")]
    InvalidPokemon(String),

    #[error("Invalid language `{0}`, should be one of [en, fr, de, ja, zh_hans, zh_hant]")]
    InvalidLanguage(String),

    #[error("Invalid generations `{0}`, should be an integers between 1 and 9")]
    InvalidGeneration(String),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
