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
