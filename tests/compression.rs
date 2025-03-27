use std::io::Cursor;

/// Tests that compressed files are read properly, by trying to read BigTest as uncompressed, gzip
/// compressed (original) and Zlib compressed and ensuring the resulting NBT of each is identical.
#[test]
fn bigtest_compression() {
    let bigtest_uncompressed = std::fs::read("tests/nbtfiles/bigtest.uncompressed.nbt").unwrap();
    let nbt_uncompressed =
        nbted::unstable::read::read_file(&mut Cursor::new(&bigtest_uncompressed)).unwrap();

    let bigtest_original = std::fs::read("tests/nbtfiles/bigtest.compressed.nbt").unwrap();
    let nbt_original =
        nbted::unstable::read::read_file(&mut Cursor::new(&bigtest_original)).unwrap();

    let bigtest_zlib = std::fs::read("tests/nbtfiles/bigtest.zlib.nbt").unwrap();
    let nbt_zlib = nbted::unstable::read::read_file(&mut Cursor::new(&bigtest_zlib)).unwrap();

    // Compare root, as the compression method in the NBTFile will differ
    assert_eq!(nbt_uncompressed.root, nbt_original.root);
    assert_eq!(nbt_uncompressed.root, nbt_zlib.root);
}
