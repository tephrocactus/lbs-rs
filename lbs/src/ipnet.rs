use super::{LBSRead, LBSWrite};
use ipnet::IpNet;
use std::{
    io::{Error, ErrorKind, Read, Result, Write},
    str::FromStr,
};

impl LBSWrite for IpNet {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        self.to_string().lbs_write(w)
    }

    #[inline]
    fn lbs_is_default(&self) -> bool {
        self.addr().lbs_is_default()
    }
}

impl LBSRead for IpNet {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self> {
        let s = String::lbs_read(r)?;
        IpNet::from_str(&s).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
}
