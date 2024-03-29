use super::LBSRead;
use super::LBSWrite;
use crate::error::LBSError;
use ipnet::IpNet;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;

impl LBSWrite for IpNet {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.to_string().lbs_write(w)
    }
}

impl LBSRead for IpNet {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let s = String::lbs_read(r)?;
        IpNet::from_str(&s).map_err(|e| LBSError::Parsing(e.to_string()))
    }
}
