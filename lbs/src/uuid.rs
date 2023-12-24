use super::LBSRead;
use super::LBSWrite;
use crate::error::LBSError;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;
use uuid::Uuid;

impl LBSWrite for Uuid {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.to_string().lbs_write(w)
    }
}

impl LBSRead for Uuid {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let s = String::lbs_read(r)?;
        Uuid::from_str(&s).map_err(|e| LBSError::Parsing(e.to_string()))
    }
}
