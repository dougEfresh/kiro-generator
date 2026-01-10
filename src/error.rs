#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("syntax error in file {0}")]
    DeserializeError(String, facet_kdl::KdlDeserializeError),
    #[error(transparent)]
    TomlDeserializeError(#[from] facet_toml::DeserializeError<facet_toml::TomlError>),
}
