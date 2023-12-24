use super::LBSRead;
use super::LBSWrite;
use crate::error::LBSError;
use std::io::Read;
use std::io::Write;
use time::OffsetDateTime;

impl LBSWrite for OffsetDateTime {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.unix_timestamp().lbs_write(w)?;
        self.nanosecond().lbs_write(w)
    }
}

impl LBSRead for OffsetDateTime {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        OffsetDateTime::from_unix_timestamp(i64::lbs_read(r)?)
            .map_err(|e| LBSError::Parsing(e.to_string()))?
            .replace_nanosecond(u32::lbs_read(r)?)
            .map_err(|e| LBSError::Parsing(e.to_string()))
    }
}
