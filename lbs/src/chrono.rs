use super::LBSRead;
use super::LBSWrite;
use crate::error::LBSError;
use chrono::prelude::*;
use std::io::Read;
use std::io::Write;

impl LBSWrite for DateTime<Utc> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.timestamp().lbs_write(w)?;
        self.timestamp_subsec_nanos().lbs_write(w)
    }
}

impl LBSRead for DateTime<Utc> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let secs = i64::lbs_read(r)?;
        let nsecs = u32::lbs_read(r)?;
        Utc.timestamp_opt(secs, nsecs)
            .single()
            .ok_or(LBSError::InvalidTimestamp)
    }
}
