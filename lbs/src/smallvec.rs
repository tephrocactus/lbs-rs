use super::{LBSRead, LBSWrite};
use smallvec::{Array, SmallVec};
use std::io::{Read, Result, Write};

impl<A> LBSWrite for SmallVec<A>
where
    A: Array,
    <A as Array>::Item: LBSWrite,
{
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        crate::write::write_len(w, self.len())?;
        for e in self {
            e.lbs_write(w)?;
        }
        Ok(())
    }

    #[inline]
    fn lbs_is_default(&self) -> bool {
        self.is_empty()
    }
}

impl<A> LBSRead for SmallVec<A>
where
    A: Array,
    <A as Array>::Item: LBSRead,
{
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self> {
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
