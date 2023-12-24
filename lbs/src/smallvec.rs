use super::LBSRead;
use super::LBSWrite;
use crate::error::LBSError;
use smallvec::Array;
use smallvec::SmallVec;
use std::io::Read;
use std::io::Write;

impl<A> LBSWrite for SmallVec<A>
where
    A: Array,
    <A as Array>::Item: LBSWrite,
{
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        crate::write::write_len(w, self.len())?;
        for e in self {
            e.lbs_write(w)?;
        }
        Ok(())
    }
}

impl<A> LBSRead for SmallVec<A>
where
    A: Array,
    <A as Array>::Item: LBSRead,
{
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let l = crate::read::read_len(r)?;

        if l == 0 {
            return Ok(Self::new());
        }

        let mut v = Self::with_capacity(l);

        for _ in 0..l {
            v.push(<A as Array>::Item::lbs_read(r)?);
        }

        Ok(v)
    }
}
