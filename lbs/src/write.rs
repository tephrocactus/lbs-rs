use num_traits::Zero;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::{Error, ErrorKind, Result},
    rc::Rc,
    sync::Arc,
};
use std::{convert::TryInto, io::Write};

pub trait LBSWrite {
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()>;
    fn lbs_omit(&self) -> bool;
}

macro_rules! impl_write_primitive {
    ($t:ty) => {
        impl LBSWrite for $t {
            #[inline]
            fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
                w.write_all(&self.to_le_bytes())
            }

            #[inline]
            fn lbs_omit(&self) -> bool {
                self.is_zero()
            }
        }
    };
}

impl_write_primitive!(u8);
impl_write_primitive!(u16);
impl_write_primitive!(u32);
impl_write_primitive!(u64);
impl_write_primitive!(usize);
impl_write_primitive!(u128);

impl_write_primitive!(i8);
impl_write_primitive!(i16);
impl_write_primitive!(i32);
impl_write_primitive!(i64);
impl_write_primitive!(i128);

impl_write_primitive!(f32);
impl_write_primitive!(f64);

impl LBSWrite for () {
    #[inline]
    fn lbs_write<W: Write>(&self, _: &mut W) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        true
    }
}

impl LBSWrite for bool {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        if *self {
            (1 as u8).lbs_write(w)
        } else {
            (0 as u8).lbs_write(w)
        }
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        *self == false
    }
}

impl LBSWrite for char {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        (*self as u32).lbs_write(w)
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        (*self as u32).lbs_omit()
    }
}

impl LBSWrite for str {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_len(w, self.len())?;
        w.write_all(self.as_bytes())
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        self.len() == 0
    }
}

impl LBSWrite for String {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        self.as_str().lbs_write(w)
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        self.len() == 0
    }
}

impl<T: LBSWrite> LBSWrite for Box<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        (**self).lbs_write(w)
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        (**self).lbs_omit()
    }
}

impl<T: LBSWrite> LBSWrite for Rc<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        (**self).lbs_write(w)
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        (**self).lbs_omit()
    }
}

impl<T: LBSWrite> LBSWrite for Arc<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        (**self).lbs_write(w)
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        (**self).lbs_omit()
    }
}

impl<'a, T: LBSWrite + ToOwned> LBSWrite for Cow<'a, T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        (**self).lbs_write(w)
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        (**self).lbs_omit()
    }
}

impl<T: LBSWrite> LBSWrite for Option<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        if let Some(v) = self {
            (1 as u8).lbs_write(w)?;
            v.lbs_write(w)
        } else {
            (0 as u8).lbs_write(w)
        }
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        self.is_none()
    }
}

impl<T: LBSWrite> LBSWrite for Vec<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_len(w, self.len())?;
        for e in self {
            e.lbs_write(w)?;
        }
        Ok(())
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        self.len() == 0
    }
}

impl<K: LBSWrite, V: LBSWrite> LBSWrite for HashMap<K, V> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_len(w, self.len())?;
        for (k, v) in self {
            k.lbs_write(w)?;
            v.lbs_write(w)?;
        }
        Ok(())
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        self.len() == 0
    }
}

impl<T: LBSWrite> LBSWrite for HashSet<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_len(w, self.len())?;
        for e in self {
            e.lbs_write(w)?;
        }
        Ok(())
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        self.len() == 0
    }
}

impl<K: LBSWrite, V: LBSWrite> LBSWrite for BTreeMap<K, V> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_len(w, self.len())?;
        for (k, v) in self {
            k.lbs_write(w)?;
            v.lbs_write(w)?;
        }
        Ok(())
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        self.len() == 0
    }
}

impl<T: LBSWrite> LBSWrite for BTreeSet<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<()> {
        write_len(w, self.len())?;
        for e in self {
            e.lbs_write(w)?;
        }
        Ok(())
    }

    #[inline]
    fn lbs_omit(&self) -> bool {
        self.len() == 0
    }
}

#[inline]
pub fn write_field_count<W: Write>(w: &mut W, count: u8) -> Result<()> {
    count.lbs_write(w)
}

#[inline]
pub fn write_field_id<W: Write>(w: &mut W, id: u8) -> Result<()> {
    id.lbs_write(w)
}

#[inline]
fn write_len<W: Write>(w: &mut W, l: usize) -> Result<()> {
    let ul: u32 = l
        .try_into()
        .map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;
    w.write_all(&ul.to_le_bytes())
}
