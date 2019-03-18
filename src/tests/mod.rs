use std::io::Cursor;

use crate::data::{Compression, NBTFile};

mod tests_data;
mod string_read;

/// Given some NBT data in original format, loops it around by converting it
/// in the following way: Read NBT -> Write String -> Read String -> Write NBT
/// and checks that the resulting NBT data is identical to the original
///
/// Obviously this won't work for compressed NBT data
fn complete_loop_from_nbt(nbt: &[u8]) {
    let mut original = Vec::new();
    original.extend_from_slice(nbt);
    let nbtfile1 = crate::read::read_file(&mut Cursor::new(original.clone()))
        .unwrap();

    let mut tmp = Vec::new();
    crate::string_write::write_file(&mut tmp, &nbtfile1).unwrap();
    let string: String = String::from_utf8(tmp).unwrap();

    let mut cursor = Cursor::new(string.into_bytes());
    let nbtfile2 = crate::string_read::read_file(&mut cursor).unwrap();

    assert_eq!(&nbtfile1, &nbtfile2);

    let mut tmp = Vec::new();
    crate::write::write_file(&mut tmp, &nbtfile2).unwrap();

    assert_eq!(original, tmp);
}

/// Given a NBTFile, loop around by converting it in the following way
/// Write String -> Read String -> Write NBT -> Read NBT and checks that the
/// resulting NBT enum is identical at each step.
fn complete_loop_from_enum(original: &NBTFile) {
    let mut tmp = Vec::new();
    crate::string_write::write_file(&mut tmp, original).unwrap();
    let string: String = String::from_utf8(tmp).unwrap();

    let mut cursor = Cursor::new(string.into_bytes());
    let nbtfile = crate::string_read::read_file(&mut cursor).unwrap();

    assert_eq!(original, &nbtfile);

    let mut tmp = Vec::new();
    crate::write::write_file(&mut tmp, &nbtfile).unwrap();

    let mut cursor = Cursor::new(tmp);
    let nbtfile = crate::read::read_file(&mut cursor).unwrap();

    assert_eq!(original, &nbtfile);
}

#[test]
fn hello_world_loop() {
    complete_loop_from_nbt(&tests_data::HELLO_WORLD);
}

#[test]
fn bigtest_uncompressed_loop() {
    complete_loop_from_nbt(&tests_data::BIGTEST_UNCOMPRESSED);
}

#[test]
fn player_file_loop() {
    complete_loop_from_nbt(&tests_data::PLAYER_FILE);
}

#[test]
fn custom_loop() {
    /* The custom file is a custom NBT file made to contain various tricky
     * edge cases that one would not normally see */
    complete_loop_from_nbt(&tests_data::CUSTOM);
}

/// Tests that we can read the original (gzip compressed) bigtest and that we
/// can loop it around correctly
#[test]
fn bigtest_original() {
    let mut data = Vec::new();
    data.extend_from_slice(&tests_data::BIGTEST_COMPRESSED);
    let mut cursor = Cursor::new(data);
    let nbtfile = crate::read::read_file(&mut cursor).unwrap();
    complete_loop_from_enum(&nbtfile);
}

/// Tests that compressed files are read properly, by trying to read BigTest
/// as uncompressed, original (gzip compressed), and Zlib compressed and
/// comparing the resulting NBT.
#[test]
fn compression_read() {
    let mut data = Vec::new();
    data.extend_from_slice(&tests_data::BIGTEST_UNCOMPRESSED);
    let mut cursor = Cursor::new(data);
    let nbt_uncompressed = crate::read::read_file(&mut cursor).unwrap();

    let mut data = Vec::new();
    data.extend_from_slice(&tests_data::BIGTEST_COMPRESSED);
    let mut cursor = Cursor::new(data);
    let nbt_gzip = crate::read::read_file(&mut cursor).unwrap();

    assert_eq!(nbt_uncompressed.root, nbt_gzip.root);

    let mut data = Vec::new();
    data.extend_from_slice(&tests_data::BIGTEST_ZLIB);
    let mut cursor = Cursor::new(data);
    let nbt_uncompressed = crate::read::read_file(&mut cursor).unwrap();

    assert_eq!(nbt_uncompressed.root, nbt_gzip.root);
}

/// Write an NBTFile to binary format, and then read it again returning the
/// result
fn write_read_binary(nbtfile: &NBTFile) -> NBTFile {
    let mut tmp = Vec::new();
    crate::write::write_file(&mut tmp, nbtfile).unwrap();
    let mut cursor = Cursor::new(tmp);
    crate::read::read_file(&mut cursor).unwrap()
}

/// Tests that files are compressed properly, by taking hello world and bigtest
/// and trying to write each of them with the two compression algorithms, and
/// then reading them back.
#[test]
fn compression_write() {
    let mut data = Vec::new();
    data.extend_from_slice(&tests_data::HELLO_WORLD);
    let mut cursor = Cursor::new(data);
    let hello_world = crate::read::read_file(&mut cursor).unwrap();

    let mut data = Vec::new();
    data.extend_from_slice(&tests_data::BIGTEST_UNCOMPRESSED);
    let mut cursor = Cursor::new(data);
    let bigtest = crate::read::read_file(&mut cursor).unwrap();

    let hello_world_gzip = NBTFile {
        root: hello_world.root.clone(),
        compression: Compression::Gzip,
    };

    let hello_world_zlib = NBTFile {
        root: hello_world.root.clone(),
        compression: Compression::Zlib,
    };

    assert_eq!(&hello_world.root,
               &write_read_binary(&hello_world_gzip).root);
    assert_eq!(&hello_world.root,
               &write_read_binary(&hello_world_zlib).root);

    let bigtest_gzip = NBTFile {
        root: bigtest.root.clone(),
        compression: Compression::Gzip,
    };

    let bigtest_zlib = NBTFile {
        root: bigtest.root.clone(),
        compression: Compression::Zlib,
    };

    assert_eq!(&bigtest.root, &write_read_binary(&bigtest_gzip).root);
    assert_eq!(&bigtest.root, &write_read_binary(&bigtest_zlib).root);
}
