pub mod domain;
pub mod models;
pub mod store;
mod schedules;
mod report;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Validation(String),
    #[error("Não foi possível acessar o banco local. Verifique o arquivo e tente novamente.")]
    Database(#[from] rusqlite::Error),
    #[error("Não foi possível acessar o arquivo. Verifique o caminho e as permissões.")]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
pub fn invalid(message: &str) -> Error { Error::Validation(message.to_owned()) }
