# LBS

[crates.io](https://crates.io/crates/lbs) | [docs.rs](https://docs.rs/lbs)

Library name stands for Lazy Binary Serialization. We call it "lazy" because it does not serizalize/deserialize struct fields initialized with default values like `0`, `0.0`, `""`, `Option::None`, empty containers and so on. This simple technique makes LBS much faster than other binary serialization libraries **when it comes to large structures**.

LBS emerged from a highload project which demands very cheap deserialization of large structures (about 160 fields) where only some fields are explicitly filled and others are initialized with default values.

## Format specification
LBS uses very simple, not self-describing format. Byte order is always little-endian.

Every struct field or enum variant is assigned with numeric ID (u16), either implicitly, using field/variant index, or explicitly, with `lbs` attribute.

Struct fields with default values are omitted during serialization/deserialization. Field may be explicitly ommited with `lbs(omit)` attribute.

During struct deserialization each field is initialized with `default()` method of it's type. Custom constructor may be explicitly defined with `lbs_default` attribute.

Type                                                 | Omitted if                     | Representation          
---------------------------------------------------- | ------------------------------ | -------------------------------
 `()`                                                | always                         | void                            
 `u{8-128}`, `i{8-128}`, `f{32-64}`, `usize`         | `0`                            | as is
 `bool`                                              | `false`                        | `u8`
 `char`                                              | `'\0'`                         | `u32`
 `String` / `str`                                    | `is_empty()`                   | length (`u32`) + content (`[u8]`)
 `std::time::Duration`                               | `as_nanos() == 0`              | secs (`u64`) + subsec_nanos (`u32`)
 `std::time::SystemTime`                             | never                          | as `Duration` since `UNIX_EPOCH`
 `std::net::ip::Ipv4Addr`                            | `is_unspecified()`             | `u32`
 `std::net::ip::Ipv6Addr`                            | `is_unspecified()`             | `u128`
 `std::net::ip::IpAddr`                              | `is_unspecified()`             | is v4 (`u8`) + `u32` or `u128`
 `std::ops::Range<T>`                                | `is_empty()`                   | start (`T`) + end (`T`)
 `Option<T>`                                         | `is_none()`                    | `0u8` or (`1u8`, `T`)
 `Box<T>` / `Rc<T>` / `Arc<T>` / `Cow<'a, T>`        | T is omitted                   | `T`
 `Vec<T>`                                            | `is_empty()`                   | length (`u32`) + content (`[T]`)
 `HashMap<K, V>` / `BTreeMap<K, V>`                  | `is_empty()`                   | length (`u32`) + content (`[(K, V)]`)
 `HashSet<T>` / `BTreeSet<T>`                        | `is_empty()`                   | length (`u32`) + content (`[T]`)
 `chrono::DateTime` (feature `chrono`)               | never                          | secs (`i64`) + subsec_nanos (`u32`)
 `smallvec::SmallVec<T>` (feature `smallvec`)        | `is_empty()`                   | length (`u32`) + content (`[T]`)
 `struct`                                            | never                          | field_count (`u8`) + field IDs and values (`[(u8, T)]`)
 `enum`                                              | never                          | variant ID (`u8`) + optional value (`T`)
 `ipnet::IpNet`                                      | `is_unspecified()`             | is v4 (`u8`) + `u32` or `u128`

## Third-party types coverage
Obviously it's impossible for library author to cover all possible types.
Currently, the only way to enable LBS for some unsupported third-party type is to use [New Type pattern](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types) and implement `LBSWrite` and `LBSRead` traits manually.

## Safety
No unsafe code is used.

## Status
Format or API changes may be introduced until v1.0.0.

## Minimal Rust Version
[1.52.1](https://blog.rust-lang.org/2021/05/10/Rust-1.52.1.html)

## Benchmark results

**Disclaimer**. Never trust third-party benchmarks. Always make your own measurements using your specific data/workload/configuration/hardware.

**Hardware**: 2,6 GHz 6-Core Intel Core i7 (12 vCPU), 16 GB 2667 MHz DDR4

**OS**: MacOS Big Sur

**Allocator**: mimalloc, without encryption

**Data**: struct with 160 fields, mostly strings. Only 20 fields are initialized with non-default values. Other fields (with default values) are ommited by serde_json, rmp_serde and LBS (speedy does not allow this).

Library    | Serialization | Deserialization | Size when serialized
---------- | ------------- | --------------- | ---------------------------
serde_json | 823.87 ns     | 2.9540 us       | 606 bytes
rmp_serde  | 550.54 ns     | 2.3190 us       | 522 bytes
speedy     | 620.72 ns     | 1.2825 us       | 944 bytes
**LBS**    | **242.62 ns** | **683.75 ns**   | **307 bytes** 

## Usage
There are `LBSWrite` and `LBSRead` traits which implementations can be derived for structs and enums.

```rust
use bytes::{Buf, BufMut, Bytes, BytesMut};
use lbs::{LBSRead, LBSWrite};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::Range,
    rc::Rc,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};

// IDs for most fields are assigned explicitly, using #[lbs(<id>)] attribute.
// Other fields receive implicit IDs (member index).
#[derive(LBSWrite, LBSRead)]
struct SomeStruct<'a> {
    #[lbs(0)]
    f0: u8,
    #[lbs(1)]
    f1: u16,
    #[lbs(2)]
    f2: u32,
    #[lbs(3)]
    f3: u64,
    #[lbs(4)]
    f4: usize,
    #[lbs(5)]
    f5: u128,
    #[lbs(6)]
    f6: i8,
    #[lbs(7)]
    f7: i16,
    #[lbs(8)]
    f8: i32,
    #[lbs(9)]
    f9: i64,
    #[lbs(10)]
    f10: f32,
    #[lbs(11)]
    f11: f64,
    #[lbs(12)]
    f12: (),
    #[lbs(13)]
    f13: bool,
    #[lbs(14)]
    f14: char,
    #[lbs(15)]
    f15: String,
    #[lbs(16)]
    f16: Duration,
    #[lbs(17)]
    #[lbs_default(SystemTime::now())]
    f17: SystemTime,
    #[lbs(18)]
    #[lbs_default(Ipv4Addr::UNSPECIFIED)]
    f18: Ipv4Addr,
    #[lbs(19)]
    #[lbs_default(Ipv6Addr::UNSPECIFIED)]
    f19: Ipv6Addr,
    #[lbs(20)]
    #[lbs_default(IpAddr::V4(Ipv4Addr::UNSPECIFIED))]
    f20: IpAddr,
    #[lbs(21)]
    #[lbs_default(Range{start:0, end:0})]
    f21: Range<u64>,
    #[lbs(22)]
    f22: Box<Vec<u64>>,
    #[lbs(23)]
    f23: Rc<String>,
    #[lbs(24)]
    f24: Arc<String>,
    #[lbs(25)]
    #[lbs_default(Arc::from(""))]
    f25: Arc<str>,
    #[lbs(26)]
    f26: Cow<'a, str>,
    #[lbs(27)]
    f27: Option<SystemTime>,
    #[lbs(28)]
    f28: Vec<String>,
    #[lbs(29)]
    f29: HashMap<String, u64>,
    #[lbs(30)]
    f30: BTreeMap<u64, String>,
    #[lbs(31)]
    f31: HashSet<String>,
    #[lbs(32)]
    f32: BTreeSet<u64>,
    #[lbs_default(chrono::Utc::now())]
    f33: chrono::DateTime<chrono::Utc>,
    f34: smallvec::SmallVec<[i64; 4]>,
    f35: AnotherStruct,
    #[lbs_default(SomeEnum::One)]
    f36: SomeEnum,
    #[lbs(omit)]
    f37: bool,
    f38: ipnet::
}

// Field IDs are assigned implicitly, using their index
#[derive(LBSWrite, LBSRead, Default)]
struct AnotherStruct {
    id: String,
    done: bool,
}

// Variant IDs are assigned implicitly, using their index
#[derive(LBSWrite, LBSRead)]
enum SomeEnum {
    One,
    Two,
    Three(String),
}

impl Default for SomeEnum {
    fn default() -> Self {
        SomeEnum::One
    }
}

#[test]
fn usage() {
    let mut original = SomeStruct {
        f0: 0,
        f1: 1,
        f2: 2,
        f3: 3,
        f4: 4,
        f5: 5,
        f6: 0,
        f7: -1,
        f8: -2,
        f9: -3,
        f10: 0.0,
        f11: -3.14,
        f12: (),
        f13: true,
        f14: 'a',
        f15: String::from("test"),
        f16: Duration::from_millis(1000),
        f17: SystemTime::now(),
        f18: Ipv4Addr::new(192, 168, 1, 2),
        f19: Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:8a2e:0370:7334").unwrap(),
        f20: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        f21: Range { start: 0, end: 1 },
        f22: Box::new(vec![1, 2, 3]),
        f23: Rc::new(String::from("test_rc")),
        f24: Arc::new(String::from("test_arc")),
        f25: Arc::from("test_str_arc"),
        f26: Cow::Owned(String::from("test_cow")),
        f27: None,
        f28: Vec::new(),
        f29: HashMap::new(),
        f30: BTreeMap::new(),
        f31: HashSet::new(),
        f32: BTreeSet::new(),
        f33: chrono::Utc::now(),
        f34: smallvec::smallvec![0, 1],
        f35: AnotherStruct::default(),
        f36: SomeEnum::Three(String::from("test_enum")),
        f37: true,
    };

    original.f29.insert(String::from("key1"), 1);
    original.f29.insert(String::from("key2"), 2);

    original.f30.insert(1, String::from("key1"));
    original.f30.insert(2, String::from("key2"));

    original.f31.insert(String::from("key1"));
    original.f31.insert(String::from("key2"));

    original.f32.insert(1);
    original.f32.insert(1);

    // Serialize
    let mut buf = Vec::with_capacity(128);
    original.lbs_write(&mut buf).unwrap();

    // Deserialize
    let decoded = SomeStruct::lbs_read(&mut buf.as_slice()).unwrap();

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
    assert_eq!(decoded.f35.id, original.f35.id);
    assert_eq!(decoded.f35.done, original.f35.done);

    if let SomeEnum::Three(s) = decoded.f36 {
        assert_eq!(s, "test_enum")
    } else {
        panic!("not SomeEnum::Three")
    }

    assert_eq!(decoded.f37, false);
}

#[test]
fn usage_batch() {
    let o1 = AnotherStruct {
        id: "1".to_string(),
        done: false,
    };

    let o2 = AnotherStruct {
        id: "2".to_string(),
        done: true,
    };

    // Serialize batch
    let batch = BytesMut::new();
    let mut w = batch.writer();
    o1.lbs_write(&mut w).unwrap();
    o2.lbs_write(&mut w).unwrap();

    // Deserialize batch
    let batch = w.into_inner();
    let mut r = batch.reader();
    let mut decoded = Vec::new();

    while r.get_ref().has_remaining() {
        decoded.push(AnotherStruct::lbs_read(&mut r).unwrap());
    }

    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded[0].id, o1.id);
    assert_eq!(decoded[0].done, o1.done);
    assert_eq!(decoded[1].id, o2.id);
    assert_eq!(decoded[1].done, o2.done);
}
```