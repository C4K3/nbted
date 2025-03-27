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

// Reads an uncompressed NBT file and then write it back as an NBT file checking that the result
// matches the original byte-for-byte
fn uncompressed_nbt_file_byte_for_byte(path: &Path) -> datatest_stable::Result<()> {
    let file = std::fs::read(path).unwrap();
    complete_loop_from_nbt(&file);
    Ok(())
}

fn nbt_file_loop(path: &Path) -> datatest_stable::Result<()> {
    let file = std::fs::read(path).unwrap();
    let mut cursor = Cursor::new(file);
    let nbtfile = nbted::unstable::read::read_file(&mut cursor).unwrap();
    complete_loop_from_enum(&nbtfile);
    Ok(())
}

datatest_stable::harness! {
    { test = uncompressed_nbt_file_byte_for_byte, root = "tests/nbtfiles", pattern = "^.*\\.uncompressed\\.nbt$" },
    { test = nbt_file_loop, root = "tests/nbtfiles" },
}
