use std::io::Cursor;

use crate::{data::NBT, unstable::tests::tests_data};

#[test]
fn single_values() {
    assert_eq!("58",                                                        serde_json::to_string(&NBT::Byte(58)).unwrap());
    assert_eq!("5348",                                                      serde_json::to_string(&NBT::Short(5348)).unwrap());
    assert_eq!("556875438",                                                 serde_json::to_string(&NBT::Long(556875438)).unwrap());
    assert_eq!("-0.345",                                                    serde_json::to_string(&NBT::Float(-0.345)).unwrap());
    assert_eq!("435.3434590872345",                                         serde_json::to_string(&NBT::Double(435.3434590872345)).unwrap());
    assert_eq!("\"Hello world\"",                                           serde_json::to_string(&NBT::String("Hello world".as_bytes().to_vec())).unwrap());
    assert_eq!("[124,51,1]",                                                serde_json::to_string(&NBT::ByteArray([124, 51, 01].to_vec())).unwrap());
    assert_eq!("[0,-51,456763]",                                            serde_json::to_string(&NBT::IntArray([0, -51, 456763].to_vec())).unwrap());
    assert_eq!("[1263435254,9223372036854775807,-9223372036854775808]",     serde_json::to_string(&NBT::LongArray([1263435254, i64::MAX, i64::MIN].to_vec())).unwrap());
    assert_eq!("[-15,\"Hi\",0.0]",                                          serde_json::to_string(&NBT::List([NBT::Byte(-15), NBT::String("Hi".as_bytes().to_vec()), NBT::Double(0.0)].to_vec())).unwrap());
    assert_eq!(r#"{"":15,"motd":"I'm a teapot!"}"#,                         serde_json::to_string(&NBT::Compound([("".as_bytes().to_vec(), NBT::Byte(15)),("motd".as_bytes().to_vec(), NBT::String("I'm a teapot!".as_bytes().to_vec()))].to_vec())).unwrap());
}

#[test]
fn nbt_file_hello_world() {
    let mut original = Vec::new();
    original.extend_from_slice(&tests_data::HELLO_WORLD);
    let nbtfile = crate::read::read_file(&mut Cursor::new(original.clone())).unwrap();

    assert_eq!(
        r#"{"root":{"hello world":{"name":"Bananrama"}},"compression":"None"}"#,
        serde_json::to_string(&nbtfile).unwrap()
    );
}

#[test]
fn nbt_file_custom() {
    let mut original = Vec::new();
    original.extend_from_slice(&tests_data::CUSTOM);
    let nbtfile = crate::read::read_file(&mut Cursor::new(original.clone())).unwrap();

    assert_eq!(
        r#"{"root":{"Root compound":{"A string with newlines in it":"Line 1\nLine 2\nLine 3","Strings can contain doublequotes":"\"Doublequoted\"","but doublequotes have to be escaped":"\"\"\n\"\nThe string didn't end until here","Names\nCan\nAlso\nBe\nMultiline":-1,"":3.14,"It shouldn't be a problem if a string ends in \\\\\\\\":[0,1,1,2,3],"Or other amounts of \\":"\\\\","Lists can contain lists":[["This is a list that contains one String. The next list is empty."],[]],"Empty ByteArray":[],"Empty IntArray":[],"Empty Compound":{}},"We can put more than one item in the implicit compound":{"Empty lists are supposed to have type End":[],"Explanation":"If empty lists have any other type they will be converted to End"},"We can also put items other than compounds in the implicit compound":1337},"compression":"None"}"#,
        serde_json::to_string(&nbtfile).unwrap()
    );
}

