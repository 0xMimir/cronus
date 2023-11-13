use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ErrorAddingJob,
    ErrorRemovingJob,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ErrorAddingJob => f.write_str("Error adding job"),
            Error::ErrorRemovingJob => f.write_str("Error removing job"),
        }
    }
}
