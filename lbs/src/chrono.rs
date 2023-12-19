use super::{LBSRead, LBSWrite};
use chrono::prelude::*;
use std::io;
use std::io::{Read, Result, Write};

impl LBSWrite for DateTime<Utc> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        self.timestamp().lbs_write(w)?;
        self.timestamp_subsec_nanos().lbs_write(w)
    }

    #[inline]
    fn lbs_is_default(&self) -> bool {
        false
    }
}

impl LBSRead for DateTime<Utc> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self> {
        let secs = i64::lbs_read(r)?;
        let nsecs = u32::lbs_read(r)?;
        Utc.timestamp_opt(secs, nsecs)
            .single()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid timestamp",
            ))
    }
}
