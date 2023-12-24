use crate::error::LBSError;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;

pub trait LBSWrite {
    fn lbs_write<W: std::io::Write>(&self, w: &mut W) -> Result<(), LBSError>;

    #[inline]
    fn lbs_must_write(&self) -> bool {
        true
    }
}

macro_rules! impl_write_primitive {
    ($t:ty) => {
        impl LBSWrite for $t {
            #[inline]
            fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
                Ok(w.write_all(&self.to_le_bytes())?)
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
impl_write_primitive!(isize);
impl_write_primitive!(i128);

impl_write_primitive!(f32);
impl_write_primitive!(f64);

impl LBSWrite for () {
    #[inline]
    fn lbs_write<W: Write>(&self, _: &mut W) -> Result<(), LBSError> {
        Ok(())
    }
}

impl<T1: LBSWrite, T2: LBSWrite> LBSWrite for (T1, T2) {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.0.lbs_write(w)?;
        self.1.lbs_write(w)
    }
}

impl<T1: LBSWrite, T2: LBSWrite, T3: LBSWrite> LBSWrite for (T1, T2, T3) {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.0.lbs_write(w)?;
        self.1.lbs_write(w)?;
        self.2.lbs_write(w)
    }
}

impl LBSWrite for bool {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        if *self {
            (1_u8).lbs_write(w)
        } else {
            (0_u8).lbs_write(w)
        }
    }
}

impl LBSWrite for char {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        (*self as u32).lbs_write(w)
    }
}

impl LBSWrite for str {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        write_len(w, self.len())?;
        Ok(w.write_all(self.as_bytes())?)
    }
}

impl LBSWrite for String {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.as_str().lbs_write(w)
    }
}

impl LBSWrite for Duration {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.as_secs().lbs_write(w)?;
        self.subsec_nanos().lbs_write(w)
    }
}

impl LBSWrite for SystemTime {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        let dur = self
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|err| Error::new(ErrorKind::Other, err))?;
        dur.lbs_write(w)
    }
}

impl LBSWrite for Ipv4Addr {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        let num: u32 = (*self).into();
        num.lbs_write(w)
    }
}

impl LBSWrite for Ipv6Addr {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        let num: u128 = (*self).into();
        num.lbs_write(w)
    }
}

impl LBSWrite for IpAddr {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        match self {
            IpAddr::V4(ip) => {
                true.lbs_write(w)?;
                ip.lbs_write(w)
            }
            IpAddr::V6(ip) => {
                false.lbs_write(w)?;
                ip.lbs_write(w)
            }
        }
    }
}

impl<T: LBSWrite + PartialOrd> LBSWrite for Range<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        self.start.lbs_write(w)?;
        self.end.lbs_write(w)
    }
}

impl<T: LBSWrite> LBSWrite for Box<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        (**self).lbs_write(w)
    }
}

impl<T: LBSWrite> LBSWrite for Rc<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        (**self).lbs_write(w)
    }
}

impl<T: LBSWrite> LBSWrite for Arc<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        (**self).lbs_write(w)
    }
}

impl<'a, T: LBSWrite + ToOwned> LBSWrite for Cow<'a, T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        (**self).lbs_write(w)
    }
}

impl<T: LBSWrite> LBSWrite for Option<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        if let Some(v) = self {
            (1_u8).lbs_write(w)?;
            v.lbs_write(w)
        } else {
            (0_u8).lbs_write(w)
        }
    }

    #[inline]
    fn lbs_must_write(&self) -> bool {
        self.is_some()
    }
}

impl<T: LBSWrite> LBSWrite for Vec<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        write_len(w, self.len())?;
        for e in self {
            e.lbs_write(w)?;
        }
        Ok(())
    }
}

impl<K: LBSWrite, V: LBSWrite> LBSWrite for HashMap<K, V> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        write_len(w, self.len())?;
        for (k, v) in self {
            k.lbs_write(w)?;
            v.lbs_write(w)?;
        }
        Ok(())
    }
}

impl<T: LBSWrite> LBSWrite for HashSet<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        write_len(w, self.len())?;
        for e in self {
            e.lbs_write(w)?;
        }
        Ok(())
    }
}

impl<K: LBSWrite, V: LBSWrite> LBSWrite for BTreeMap<K, V> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        write_len(w, self.len())?;
        for (k, v) in self {
            k.lbs_write(w)?;
            v.lbs_write(w)?;
        }
        Ok(())
    }
}

impl<T: LBSWrite> LBSWrite for BTreeSet<T> {
    #[inline]
    fn lbs_write<W: Write>(&self, w: &mut W) -> Result<(), LBSError> {
        write_len(w, self.len())?;
        for e in self {
            e.lbs_write(w)?;
        }
        Ok(())
    }
}

#[inline]
pub fn write_field_count<W: Write>(w: &mut W, count: u16) -> Result<(), LBSError> {
    count.lbs_write(w)
}

#[inline]
pub fn write_field_id<W: Write>(w: &mut W, id: u16) -> Result<(), LBSError> {
    id.lbs_write(w)
}

#[inline]
pub fn write_len<W: Write>(w: &mut W, l: usize) -> Result<(), LBSError> {
    let ul: u32 = l
        .try_into()
        .map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;
    Ok(w.write_all(&ul.to_le_bytes())?)
}
