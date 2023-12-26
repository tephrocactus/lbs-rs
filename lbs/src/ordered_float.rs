use crate::error::LBSError;
use crate::LBSRead;
use crate::LBSWrite;
use ordered_float::OrderedFloat;
use std::io::Read;
use std::io::Write;

impl<T: LBSWrite> LBSWrite for OrderedFloat<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.0.lbs_write(w)
    }
}

impl<T: LBSRead> LBSRead for OrderedFloat<T> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(OrderedFloat(T::lbs_read(r)?))
    }
}
