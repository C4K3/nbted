/// Various testing of the reading of files in the pretty text format into
/// binary NBT files. This is only used for testing abnormally formatted but
/// otherwise valid text files, and for testing failure states of the string
/// reading.
///
/// Testing of valid and regularly formatted files are conducted in the main
/// file with its "loops".

use std::io::Cursor;

use error_chain::ChainedError;

use crate::data::NBTFile;
use crate::errors::Result;

/// Convenience method
fn try_parse_string(original: &str) -> Result<NBTFile> {
    let mut cursor = Cursor::new(original.as_bytes());
    crate::string_read::read_file(&mut cursor)
}

fn try_parse_string_get_err_msg(original: &str) -> String {
    let err = match try_parse_string(original) {
        Ok(_) => panic!("try_parse_string_get_err_msg test expected the file to be Err but it was Ok"),
        Err(e) => e,
    };
    format!("{}", err.display_chain())
}

#[test]
fn empty_file() {
    let err_msg = try_parse_string_get_err_msg("   ");
    assert!(err_msg.contains("NBT file in text format does not contain any tags at all"));
}

#[test]
fn text_file_with_no_trailing_bytes() {
    /* I experienced an error with the new tokenizer where it would ignore the
     * last tag unless there was at least 1 byte following it */
    let _: NBTFile = try_parse_string("None End").unwrap();
}

#[test]
fn incomplete_string() {
    let err_msg = try_parse_string_get_err_msg(r#"None Compound "A quotation mark at the end has been removed from this otherwise valid NBT file End End"#);
    assert!(err_msg.contains("EOF when trying to read the name of a Compound tag in a compound"));
}

#[test]
fn escaped_unescapable_char() {
    let err_msg = try_parse_string_get_err_msg(r#"None Compound "\k" End End"#);
    assert!(err_msg.contains("Invalid string, tried to escape the character"));
}

#[test]
fn eof_when_reading() {
    let err_msg = try_parse_string_get_err_msg(r#"None Compound "" Short """#);
    assert!(err_msg.contains("EOF when trying to read a short"));
}

#[test]
fn invalid_int() {
    let err_msg = try_parse_string_get_err_msg(r#"Zlib Compound "" Int "" NotAnInt End End"#);
    assert!(err_msg.contains("Invalid Int NotAnInt"));
}

#[test]
fn invalid_tag_type() {
    let err_msg = try_parse_string_get_err_msg(r#"Gzip Compound "" List "" NotATagType 1 9 End End"#);
    assert!(err_msg.contains("Unknown tag type NotATagType"));
}

#[test]
fn unquoted_string() {
    /* Since the rewrite of the tokenizer, strings without quotation marks have
     * become valid syntax. I'm not sure if I want to keep this valid or not,
     * so I'll leave it undocumented except for this test testing that they are
     * parsed */
    let _: NBTFile = try_parse_string(r#"None Compound ForgotQuotationMarksAroundThisString End End"#).unwrap();
}
