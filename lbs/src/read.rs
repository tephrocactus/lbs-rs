use crate::error::LBSError;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::io::Read;
use std::mem::size_of;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;

pub trait LBSRead: Sized {
    fn lbs_read<R: std::io::Read>(r: &mut R) -> Result<Self, LBSError>;
}

macro_rules! impl_read_primitive {
    ($t:ty, $l:expr) => {
        impl LBSRead for $t {
            #[inline]
            fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
                let mut buf = [0; $l];
                r.read_exact(&mut buf)?;
                Ok(Self::from_le_bytes(buf))
            }
        }
    };
}

impl_read_primitive!(u8, 1);
impl_read_primitive!(u16, 2);
impl_read_primitive!(u32, 4);
impl_read_primitive!(u64, 8);
impl_read_primitive!(usize, 8);
impl_read_primitive!(u128, 16);

impl_read_primitive!(i8, 1);
impl_read_primitive!(i16, 2);
impl_read_primitive!(i32, 4);
impl_read_primitive!(i64, 8);
impl_read_primitive!(isize, 8);
impl_read_primitive!(i128, 16);

impl_read_primitive!(f32, 4);
impl_read_primitive!(f64, 8);

impl LBSRead for () {
    #[inline]
    fn lbs_read<R: Read>(_: &mut R) -> Result<Self, LBSError> {
        Ok(())
    }
}

impl<T1: LBSRead, T2: LBSRead> LBSRead for (T1, T2) {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok((T1::lbs_read(r)?, T2::lbs_read(r)?))
    }
}

impl<T1: LBSRead, T2: LBSRead, T3: LBSRead> LBSRead for (T1, T2, T3) {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok((T1::lbs_read(r)?, T2::lbs_read(r)?, T3::lbs_read(r)?))
    }
}

impl LBSRead for bool {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let mut buf = [0; 1];
        r.read_exact(&mut buf)?;
        Ok(u8::from_le_bytes(buf) == 1)
    }
}

impl LBSRead for char {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let mut buf = [0; 4];
        r.read_exact(&mut buf)?;
        Self::from_u32(u32::from_le_bytes(buf)).ok_or_else(|| LBSError::InvalidChar)
    }
}

impl LBSRead for String {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let l = read_len(r)?;

        if l == 0 {
            return Ok(Self::new());
        }

        let mut buf = vec![0; l];
        r.read_exact(&mut buf)?;
        Self::from_utf8(buf).map_err(|e| LBSError::Parsing(e.to_string()))
    }
}

impl LBSRead for Duration {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let secs = u64::lbs_read(r)?;
        let nanos = u32::lbs_read(r)?;
        Ok(Self::new(secs, nanos))
    }
}

impl LBSRead for SystemTime {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Self::UNIX_EPOCH
            .checked_add(Duration::lbs_read(r)?)
            .ok_or_else(|| LBSError::InvalidTimestamp)
    }
}

impl LBSRead for Ipv4Addr {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(Self::from(u32::lbs_read(r)?))
    }
}

impl LBSRead for Ipv6Addr {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(Self::from(u128::lbs_read(r)?))
    }
}

impl LBSRead for IpAddr {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(match bool::lbs_read(r)? {
            true => IpAddr::V4(Ipv4Addr::lbs_read(r)?),
            false => IpAddr::V6(Ipv6Addr::lbs_read(r)?),
        })
    }
}

impl<T: LBSRead + PartialOrd> LBSRead for Range<T> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let start = T::lbs_read(r)?;
        let end = T::lbs_read(r)?;
        Ok(Range { start, end })
    }
}

impl<T: LBSRead> LBSRead for Box<T> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(Self::new(T::lbs_read(r)?))
    }
}

impl<T: LBSRead> LBSRead for Rc<T> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(Self::new(T::lbs_read(r)?))
    }
}

impl<T: LBSRead> LBSRead for Arc<T> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(Self::new(T::lbs_read(r)?))
    }
}

impl LBSRead for Arc<str> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(Self::from(String::lbs_read(r)?))
    }
}

impl<'a, T: LBSRead + ToOwned> LBSRead for Cow<'a, T> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(Self::Owned(T::lbs_read(r)?.to_owned()))
    }
}

impl<'a> LBSRead for Cow<'a, str> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        Ok(Self::Owned(String::lbs_read(r)?))
    }
}

impl<T: LBSRead> LBSRead for Option<T> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let mut buf = [0; 1];
        r.read_exact(&mut buf)?;
        if u8::from_le_bytes(buf) == 1 {
            Ok(Some(T::lbs_read(r)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: LBSRead> LBSRead for Vec<T> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let l = read_len(r)?;

        if l == 0 {
            return Ok(Self::new());
        }

        let mut v = Self::with_capacity(l);

        for _ in 0..l {
            v.push(T::lbs_read(r)?);
        }

        Ok(v)
    }
}

impl<K, V, S> LBSRead for HashMap<K, V, S>
where
    K: LBSRead + Eq + Hash,
    V: LBSRead,
    S: BuildHasher + Default,
{
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let l = read_len(r)?;

        if l == 0 {
            return Ok(Self::default());
        }

        let mut hm = Self::with_capacity_and_hasher(l, S::default());

        for _ in 0..l {
            let k = K::lbs_read(r)?;
            let v = V::lbs_read(r)?;
            hm.insert(k, v);
        }

        Ok(hm)
    }
}

impl<K, S> LBSRead for HashSet<K, S>
where
    K: LBSRead + Eq + Hash,
    S: BuildHasher + Default,
{
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let l = read_len(r)?;

        if l == 0 {
            return Ok(Self::default());
        }

        let mut hs = Self::with_capacity_and_hasher(l, S::default());

        for _ in 0..l {
            hs.insert(K::lbs_read(r)?);
        }

        Ok(hs)
    }
}

impl<K: LBSRead + Ord, V: LBSRead> LBSRead for BTreeMap<K, V> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let l = read_len(r)?;
        let mut bm = Self::new();

        if l == 0 {
            return Ok(bm);
        }

        for _ in 0..l {
            let k = K::lbs_read(r)?;
            let v = V::lbs_read(r)?;
            bm.insert(k, v);
        }

        Ok(bm)
    }
}

impl<K: LBSRead + Ord> LBSRead for BTreeSet<K> {
    #[inline]
    fn lbs_read<R: Read>(r: &mut R) -> Result<Self, LBSError> {
        let l = read_len(r)?;
        let mut bm = Self::new();

        if l == 0 {
            return Ok(bm);
        }

        for _ in 0..l {
            bm.insert(K::lbs_read(r)?);
        }

        Ok(bm)
    }
}

#[inline]
pub fn read_field_count<R: Read>(r: &mut R) -> Result<u16, LBSError> {
    u16::lbs_read(r)
}

#[inline]
pub fn read_field_id<R: Read>(r: &mut R) -> Result<u16, LBSError> {
    u16::lbs_read(r)
}

#[inline]
pub fn read<T: LBSRead, R: Read>(r: &mut R) -> Result<T, LBSError> {
    T::lbs_read(r)
}

#[inline]
pub fn read_len<R: Read>(r: &mut R) -> Result<usize, LBSError> {
    let mut buf = [0; size_of::<u32>()];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf) as usize)
}
