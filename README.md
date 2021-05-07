# LBS
Library name stands for Lazy Binary Serialization. We call it "lazy" because it does not serizalize/deserialize struct fields initialized with default values like `0`, `0.0`, `""`, `Option::None`, empty containers and so on. This simple technique makes LBS much faster than other binary serialization libraries **when it comes to large structures**.

LBS emerged from a highload project which demands very cheap deserialization of large structures (about 160 fields) where only some fields are explicitly filled and others are initialized with default values.

## Why no serde?
[Serde](https://github.com/serde-rs/serde) is great and convenient framework, but, being an additional layer between user's and serialization library's code, it incurs substantial performance penalty. Moreover, serde 1.0.125 still does not allow to use numeric values as field names (or identifiers) which is another performance penalty if we only want to serialize some fields and omit others.

## Why not just use other libraries without serde?
Indeed, there are some binary serialization libraries out there which do not require serde. Good examples are [speedy](https://github.com/koute/speedy) and [borsh](https://github.com/near/borsh-rs), but, unfortunately, they do not support conditional serialization - every field must be serialized/deserialized. There is also [msgpack-rust](https://github.com/3Hren/msgpack-rust) (rmp) which does support conditional serialization as maps, but it is still not fast enough for our use case.


## Format specification
LBS uses very simple, not self-describing format. Byte order is always little-endian.

Every struct field or enum variant is assigned with numeric ID, either implicitly, using field/variant index, or explicitly, with `lbs` attribute. IDs are represented as u8, so maximum number of fields/variants is 255.

Struct fields with default values are ommited during serialization/deserialization. Field may be explicitly ommited with `lbs(omit)` attribute.

During struct deserialization each field is constructed wtih `default()` method of it's type. Custom constructor may be explicitly defined with `lbs_default` attribute.

Type                                          | Representation          
--------------------------------------------- | -------------------------------
 `()`                                         | void
 `u{8-128}`, `i{8-128}`, `f{32-64}`, `usize`  | as is
 `bool`                                       | `u8`
 `char`                                       | `u32`
 `String` / `str`                             | length (`u32`) + content (`[u8]`)
 `Option<T>`                                  | `0u8` or (`1u8`, `T`)
 `Box<T>` / `Rc<T>` / `Arc<T>` / `Cow<'a, T>`  | `T`
 `Vec<T>`                                     | length (`u32`) + content (`[T]`)
 `HashMap<K, V>` / `BTreeMap<K, V>`           | length (`u32`) + content (`[(K, V)]`)
 `HashSet<T>` / `BTreeSet<T>`                 | length (`u32`) + content (`[T]`)
 `struct`                                     | field_count (`u8`) + field IDs and values (`[(u8, T)]`)
 `enum`                                       | variant ID (`u8`) + optional value (`T`)

## Safety
No unsafe code is used.

## Status
Format or API changes may be introduced until v1.0.0.

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
**LBS**    | **242.62 ns** | **683.75 ns**   | **287 bytes** 

## Usage example
There are `LBSWrite` and `LBSRead` traits which implementations can be derived for structs and enums.

```rust
use lbs::{LBSRead, LBSWrite};
use std::sync::Arc;

#[derive(LBSWrite, LBSRead)]
struct SomeStruct {
    #[lbs(0)]
    f0: u8,
    #[lbs(1)]
    f1: u64,
    #[lbs(2)]
    #[lbs_default(Arc::from(""))]
    f2: Arc<str>,
    #[lbs(3)]
    #[lbs_default(SomeEnum::One)]
    f3: SomeEnum,
    #[lbs(4)]
    f4: Option<SomeOptionalStruct>,
    #[lbs(omit)]
    _omitted: bool,
}

#[derive(LBSWrite, LBSRead)]
struct SomeOptionalStruct {
    id: String,
    done: bool,
}

#[derive(LBSWrite, LBSRead)]
enum SomeEnum {
    One,
    Two,
    Three(String),
}

#[test]
fn usage() {
    let original = SomeStruct {
        f0: 0,
        f1: 1,
        f2: Arc::from(""),
        f3: SomeEnum::Three(String::from("test")),
        f4: None,
        _omitted: true,
    };

    let mut buf = Vec::with_capacity(128);
    original.lbs_write(&mut buf).unwrap();
    let decoded = SomeStruct::lbs_read(&mut buf.as_slice()).unwrap();

    assert_eq!(decoded.f0, original.f0);
    assert_eq!(decoded.f1, original.f1);
    assert_eq!(decoded.f2, original.f2);

    if let SomeEnum::Three(s) = decoded.f3 {
        assert_eq!(s, "test")
    } else {
        panic!("not SomeEnum::Three")
    }

    assert!(decoded.f4.is_none());
    assert_eq!(decoded._omitted, false);
}
```