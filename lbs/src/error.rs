use thiserror::Error;

#[derive(Error, Debug)]
pub enum LBSError {
    #[error("required but missing")]
    RequiredButMissing,
    #[error("unexpected enum variant")]
    UnexpectedVariant,
    #[error("invalid timestamp")]
    InvalidTimestamp,
    #[error("invalid char")]
    InvalidChar,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parsing(String),
    #[error("field {0}: {1}")]
    WithField(u16, Box<LBSError>),
}

impl LBSError {
    pub fn is_eof(&self) -> bool {
        match self {
            Self::Io(e) => e.kind() == std::io::ErrorKind::UnexpectedEof,
            Self::WithField(_, e) => e.is_eof(),
            _ => false,
        }
    }

    pub fn with_field(self, field_id: u16) -> Self {
        match self {
            Self::WithField(..) => self,
            other => Self::WithField(field_id, other.into()),
        }
    }
}
