use super::LBSRead;
use super::LBSWrite;
use crate::error::LBSError;
use fraction::Decimal;
use fraction::Fraction;
use std::io::Read;
use std::io::Write;

impl LBSWrite for Fraction {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.to_string().lbs_write(w)
    }
}

impl LBSWrite for Decimal {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.to_string().lbs_write(w)
    }
}

impl LBSRead for Fraction {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        String::lbs_read(r)?
            .parse::<Self>()
            .map_err(|e| LBSError::Parsing(e.to_string()))
    }
}

impl LBSRead for Decimal {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        String::lbs_read(r)?
            .parse::<Self>()
            .map_err(|e| LBSError::Parsing(e.to_string()))
    }
}
