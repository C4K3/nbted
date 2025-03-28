use std::io::Cursor;
use std::path::Path;

use nbted::unstable::data::NBTFile;

/// Given some NBT data in original format, loops it around by converting it
/// in the following way: Read NBT -> Write String -> Read String -> Write NBT
/// and checks that the resulting NBT data is identical to the original
///
/// Obviously this won't work for compressed NBT data because it'll write it
fn complete_loop_from_nbt(nbt: &[u8]) {
    let mut original = Vec::new();
    original.extend_from_slice(nbt);
    let nbtfile1 = nbted::unstable::read::read_file(&mut Cursor::new(original.clone())).unwrap();

    let mut tmp = Vec::new();
    nbted::unstable::string_write::write_file(&mut tmp, &nbtfile1).unwrap();
    let string: String = String::from_utf8(tmp).unwrap();

    let mut cursor = Cursor::new(string.into_bytes());
    let nbtfile2 = nbted::unstable::string_read::read_file(&mut cursor).unwrap();

    assert_eq!(&nbtfile1, &nbtfile2);

    let mut tmp = Vec::new();
    nbted::unstable::write::write_file(&mut tmp, &nbtfile2).unwrap();

    assert_eq!(original, tmp);
}

/// Given a NBTFile, loop around by converting it in the following way
/// Write String -> Read String -> Write NBT -> Read NBT and checks that the
/// resulting NBT enum is identical at each step.
fn complete_loop_from_enum(original: &NBTFile) {
    let mut tmp = Vec::new();
    nbted::unstable::string_write::write_file(&mut tmp, original).unwrap();
    let string: String = String::from_utf8(tmp).unwrap();

    let mut cursor = Cursor::new(string.into_bytes());
    let nbtfile = nbted::unstable::string_read::read_file(&mut cursor).unwrap();

    assert_eq!(original, &nbtfile);

    let mut tmp = Vec::new();
    nbted::unstable::write::write_file(&mut tmp, &nbtfile).unwrap();

    let mut cursor = Cursor::new(tmp);
    let nbtfile = nbted::unstable::read::read_file(&mut cursor).unwrap();

    assert_eq!(original, &nbtfile);
}

/// Reads an uncompressed NBT file and then write it back as an NBT file checking that the result
/// matches the original byte-for-byte
fn uncompressed_nbt_file_byte_for_byte(path: &Path) -> datatest_stable::Result<()> {
    let file = std::fs::read(path).unwrap();
    complete_loop_from_nbt(&file);
    Ok(())
}

fn nbt_file_decode_encode(path: &Path) -> datatest_stable::Result<()> {
    let file = std::fs::read(path).unwrap();
    let mut cursor = Cursor::new(file);
    let nbtfile = nbted::unstable::read::read_file(&mut cursor).unwrap();
    complete_loop_from_enum(&nbtfile);
    Ok(())
}

/// Tests reading and then re-writing nbted-format text files, ensuring they're encoding
/// identically.
fn txt_file_decode_encode(path: &Path) -> datatest_stable::Result<()> {
    let file = std::fs::read(path).unwrap();
    let nbtfile = nbted::unstable::string_read::read_file(&mut Cursor::new(&file)).unwrap();
    let mut encoded = Vec::with_capacity(file.len());
    nbted::unstable::string_write::write_file(&mut encoded, &nbtfile).unwrap();
    assert_eq!(file, encoded);
    Ok(())
}

/// Compares the decoding of given nbt files with corresponding saved txt files.
///
/// Each file in `nbtfiles` is expected to have a corresponding txt file in `string_write`. Why
/// string_write? Because we expect the files used for comparison to be encoded in default
/// settings.
fn compare_nbt_txt(path: &Path) -> datatest_stable::Result<()> {
    let nbt = {
        let file = std::fs::read(path).unwrap();
        nbted::unstable::read::read_file(&mut Cursor::new(&file)).unwrap()
    };

    let txt = {
        let mut txt_path = path.to_path_buf();
        let file_name = path.file_name().unwrap();
        txt_path.pop();
        txt_path.pop();
        txt_path.push("string_write");
        txt_path.push(file_name);
        txt_path.set_extension("txt");

        let file = std::fs::read(&txt_path).unwrap();
        nbted::unstable::string_read::read_file(&mut Cursor::new(&file)).unwrap()
    };

    assert_eq!(nbt, txt);

    Ok(())
}

/// Tests that the given txt files can be read, but if written again it won't necessarily match the
/// input.
fn string_read(path: &Path) -> datatest_stable::Result<()> {
    let file = std::fs::read(path).unwrap();
    nbted::unstable::string_read::read_file(&mut Cursor::new(&file)).unwrap();
    Ok(())
}

/// Tries to read invalid txt files and compares the resulting error against an expected error.
fn string_read_err(path: &Path) -> datatest_stable::Result<()> {
    let file = std::fs::read(path).unwrap();
    let err = nbted::unstable::string_read::read_file(&mut Cursor::new(&file)).unwrap_err();

    let expected_err: String = {
        let mut err_path = path.to_path_buf();
        err_path.set_extension("err");
        std::fs::read_to_string(err_path).unwrap()
    };

    assert_eq!(err.to_string(), expected_err);

    Ok(())
}

datatest_stable::harness! {
    { test = uncompressed_nbt_file_byte_for_byte, root = "tests/nbtfiles", pattern = "^.*\\.uncompressed\\.nbt$" },
    { test = nbt_file_decode_encode, root = "tests/nbtfiles" },
    { test = txt_file_decode_encode, root = "tests/string_write" },
    { test = compare_nbt_txt, root = "tests/nbtfiles" },
    { test = string_read, root = "tests/string_read" },
    { test = string_read_err, root = "tests/string_read_err", pattern = "^.*\\.txt" },
}
