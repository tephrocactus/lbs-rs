#![allow(unused_imports, dead_code)]

use bytes::Buf;
use bytes::BufMut;
use bytes::Bytes;
use bytes::BytesMut;
use chrono::NaiveDate;
use fraction::Decimal;
use fraction::Fraction;
use ipnet::IpNet;
use lbs::error::LBSError;
use lbs::LBSRead;
use lbs::LBSWrite;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::ops::Range;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(LBSWrite, LBSRead)]
struct StructOne<'a> {
    #[lbs(id(0))]
    f0: u8,
    #[lbs(id(1))]
    f1: u16,
    #[lbs(id(2))]
    f2: u32,
    #[lbs(id(3))]
    f3: u64,
    #[lbs(id(4))]
    f4: usize,
    #[lbs(id(5))]
    f5: u128,
    #[lbs(id(6))]
    f6: i8,
    #[lbs(id(7))]
    f7: i16,
    #[lbs(id(8))]
    f8: i32,
    #[lbs(id(9))]
    f9: i64,
    #[lbs(id(10))]
    f10: isize,
    #[lbs(id(11))]
    f11: i128,
    #[lbs(id(12))]
    f12: f32,
    #[lbs(id(13))]
    f13: f64,
    #[lbs(id(14))]
    f14: (),
    #[lbs(id(15))]
    f15: (u64, String),
    #[lbs(id(16))]
    f16: (u64, u64, u64),
    #[lbs(id(17))]
    f17: bool,
    #[lbs(id(18))]
    f18: char,
    #[lbs(id(19))]
    f19: String,
    #[lbs(id(20))]
    f20: Duration,
    #[lbs(id(21), default(SystemTime::UNIX_EPOCH))]
    f21: SystemTime,
    #[lbs(id(22), default(Ipv4Addr::UNSPECIFIED))]
    f22: Ipv4Addr,
    #[lbs(id(23), default(Ipv6Addr::UNSPECIFIED))]
    f23: Ipv6Addr,
    #[lbs(id(24), default(IpAddr::V4(Ipv4Addr::UNSPECIFIED)))]
    f24: IpAddr,
    #[lbs(id(25))]
    f25: Range<u64>,
    #[lbs(id(26))]
    f26: Vec<u64>,
    #[lbs(id(27))]
    f27: Rc<String>,
    #[lbs(id(28))]
    f28: Arc<String>,
    #[lbs(id(29), default(Arc::from("")))]
    f29: Arc<str>,
    #[lbs(id(30))]
    f30: Cow<'a, str>,
    #[lbs(id(31))]
    f31: Option<String>,
    #[lbs(id(32))]
    f32: Vec<String>,
    #[lbs(id(33))]
    f33: HashMap<String, u64>,
    #[lbs(id(34))]
    f34: BTreeMap<u64, String>,
    #[lbs(id(35))]
    f35: HashSet<String>,
    #[lbs(id(36))]
    f36: BTreeSet<u64>,
    #[lbs(id(37))]
    f37: chrono::DateTime<chrono::Utc>,
    #[lbs(id(38))]
    f38: smallvec::SmallVec<[i64; 4]>,
    #[lbs(id(39))]
    f39: StructTwo,
    #[lbs(id(40))]
    f40: EnumOne,
    #[lbs(id(41))]
    f41: IpNet,
    #[lbs(id(42))]
    f42: Uuid,
    #[lbs(id(43), default(OffsetDateTime::UNIX_EPOCH))]
    f43: OffsetDateTime,
    #[lbs(id(44), skip)]
    f44: bool,
    #[lbs(id(45))]
    f45: Fraction,
    #[lbs(id(46))]
    f46: Decimal,
    #[lbs(id(47))]
    f47: Cow<'a, [String]>,
}

// Field IDs are assigned implicitly, using their index
#[derive(LBSWrite, LBSRead, Default, PartialEq, Debug)]
struct StructTwo {
    #[lbs(id(0))]
    id: Uuid,
    #[lbs(id(1))]
    name: String,
    #[lbs(id(2))]
    en: Option<EnumOne>,
}

// Variant IDs are assigned implicitly, using their index
#[derive(LBSWrite, LBSRead, PartialEq, Debug, Default)]
enum EnumOne {
    #[default]
    #[lbs(id(0))]
    One,
    #[lbs(id(1))]
    Two,
    #[lbs(id(2))]
    Three(String),
    #[lbs(id(3))]
    Four(EnumTwo),
}

#[derive(LBSWrite, LBSRead, PartialEq, Debug)]
enum EnumTwo {
    #[lbs(id(0))]
    One,
    #[lbs(id(1))]
    Two,
}

#[test]
fn usage() {
    let mut original = StructOne {
        f0: 1,
        f1: 1,
        f2: 2,
        f3: 3,
        f4: 4,
        f5: 5,
        f6: 1,
        f7: -1,
        f8: -2,
        f9: -3,
        f10: 23,
        f11: 1,
        f12: 1.1,
        f13: -3.14,
        f14: (),
        f15: (1, String::from("1")),
        f16: (1, 2, 3),
        f17: true,
        f18: 'a',
        f19: String::from("test"),
        f20: Duration::from_millis(1000),
        f21: SystemTime::now(),
        f22: Ipv4Addr::new(192, 168, 1, 2),
        f23: Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:8a2e:0370:7334").unwrap(),
        f24: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        f25: Range { start: 0, end: 1 },
        f26: vec![1, 2, 3],
        f27: Rc::new(String::from("test_rc")),
        f28: Arc::new(String::from("test_arc")),
        f29: Arc::from("test_str_arc"),
        f30: Cow::Owned(String::from("test_cow")),
        f31: Some("str".to_string()),
        f32: Vec::new(),
        f33: HashMap::new(),
        f34: BTreeMap::new(),
        f35: HashSet::new(),
        f36: BTreeSet::new(),
        f37: chrono::Utc::now(),
        f38: smallvec::smallvec![0, 1],
        f39: StructTwo::default(),
        f40: EnumOne::Three(String::from("test_enum")),
        f41: IpNet::from_str("192.168.1.0/24").unwrap(),
        f42: Uuid::new_v4(),
        f43: OffsetDateTime::now_utc(),
        f44: true,
        f45: Fraction::from(3.14),
        f46: Decimal::from(3.15),
        f47: vec!["a".to_string(), "b".to_string(), "c".to_string()].into(),
    };

    original.f33.insert(String::from("key1"), 1);
    original.f33.insert(String::from("key2"), 2);

    original.f34.insert(1, String::from("key1"));
    original.f34.insert(2, String::from("key2"));

    original.f35.insert(String::from("key1"));
    original.f35.insert(String::from("key2"));

    original.f36.insert(1);
    original.f36.insert(1);

    let mut buf = Vec::with_capacity(128);
    original.lbs_write(&mut buf).unwrap();

    let decoded = StructOne::lbs_read(&mut buf.as_slice()).unwrap();
    assert_eq!(decoded.f0, original.f0);
    assert_eq!(decoded.f1, original.f1);
    assert_eq!(decoded.f2, original.f2);
    assert_eq!(decoded.f3, original.f3);
    assert_eq!(decoded.f4, original.f4);
    assert_eq!(decoded.f5, original.f5);
    assert_eq!(decoded.f6, original.f6);
    assert_eq!(decoded.f7, original.f7);
    assert_eq!(decoded.f8, original.f8);
    assert_eq!(decoded.f9, original.f9);
    assert_eq!(decoded.f10, original.f10);
    assert_eq!(decoded.f11, original.f11);
    assert_eq!(decoded.f12, original.f12);
    assert_eq!(decoded.f13, original.f13);
    assert_eq!(decoded.f14, original.f14);
    assert_eq!(decoded.f15, original.f15);
    assert_eq!(decoded.f16, original.f16);
    assert_eq!(decoded.f17, original.f17);
    assert_eq!(decoded.f18, original.f18);
    assert_eq!(decoded.f19, original.f19);
    assert_eq!(decoded.f20, original.f20);
    assert_eq!(decoded.f21, original.f21);
    assert_eq!(decoded.f22, original.f22);
    assert_eq!(decoded.f23, original.f23);
    assert_eq!(decoded.f24, original.f24);
    assert_eq!(decoded.f25, original.f25);
    assert_eq!(decoded.f26, original.f26);
    assert_eq!(decoded.f27, original.f27);
    assert_eq!(decoded.f28, original.f28);
    assert_eq!(decoded.f29, original.f29);
    assert_eq!(decoded.f30, original.f30);
    assert_eq!(decoded.f31, original.f31);
    assert_eq!(decoded.f32, original.f32);
    assert_eq!(decoded.f33, original.f33);
    assert_eq!(decoded.f34, original.f34);
    assert_eq!(decoded.f35, original.f35);
    assert_eq!(decoded.f36, original.f36);
    assert_eq!(decoded.f37, original.f37);
    assert_eq!(decoded.f38, original.f38);
    assert_eq!(decoded.f39, original.f39);
    assert_eq!(decoded.f40, original.f40);
    assert_eq!(decoded.f41, original.f41);
    assert_eq!(decoded.f42, original.f42);
    assert_eq!(decoded.f43, original.f43);
    assert_eq!(decoded.f44, false);
    assert_eq!(decoded.f45, original.f45);
    assert_eq!(decoded.f46, original.f46);
    assert_eq!(decoded.f47, original.f47);
}

#[derive(LBSWrite, LBSRead, PartialEq, Debug)]
struct MessageV1 {
    #[lbs(id(0))]
    f0: u64,
    #[lbs(id(1))]
    f1: Option<u64>,
}

#[derive(LBSWrite, LBSRead, PartialEq, Debug)]
struct MessageV2 {
    #[lbs(id(0))]
    f0: u64,
    #[lbs(id(1))]
    f1: Option<u64>,
    #[lbs(id(2))]
    f2: u64,
}

#[test]
fn required() {
    let msgv1 = MessageV1 { f0: 1, f1: None };

    let mut buf = Vec::with_capacity(128);
    msgv1.lbs_write(&mut buf).unwrap();

    if let Err(e) = MessageV2::lbs_read(&mut buf.as_slice()) {
        if let LBSError::WithField(id, inner) = e {
            if let LBSError::RequiredButMissing = inner.as_ref() {
                if id == 2 {
                    return;
                }
            }
        }
    }

    panic!("not an error");
}

#[derive(LBSWrite, LBSRead, PartialEq, Debug)]
struct OtherMessageV1 {
    #[lbs(id(0))]
    f0: u64,
    #[lbs(id(1))]
    f1: Option<u64>,
}

#[derive(LBSWrite, LBSRead, PartialEq, Debug)]
struct OtherMessageV2 {
    #[lbs(id(0))]
    f0: u64,
    #[lbs(id(1))]
    f1: Option<u64>,
    #[lbs(id(2), optional)]
    f2: u64,
}

#[test]
fn optional() {
    let msgv1 = OtherMessageV1 { f0: 1, f1: None };
    let mut buf = Vec::with_capacity(128);
    msgv1.lbs_write(&mut buf).unwrap();
    OtherMessageV2::lbs_read(&mut buf.as_slice()).unwrap();
}
