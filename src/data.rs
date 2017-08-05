/// Represents a single NBT tab
#[derive(Clone, PartialEq, Debug)]
pub enum NBT {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NBT>),
    Compound(Vec<(String, NBT)>),
    IntArray(Vec<i32>),
}
impl NBT {
    /// Returns the type of the tag as an English string
    pub fn type_string(&self) -> &str {
        match self {
            &NBT::End => "End",
            &NBT::Byte(..) => "Byte",
            &NBT::Short(..) => "Short",
            &NBT::Int(..) => "Int",
            &NBT::Long(..) => "Long",
            &NBT::Float(..) => "Float",
            &NBT::Double(..) => "Double",
            &NBT::ByteArray(..) => "ByteArray",
            &NBT::String(..) => "String",
            &NBT::List(..) => "List",
            &NBT::Compound(..) => "Compound",
            &NBT::IntArray(..) => "IntArray",
        }
    }
    /// Returns the type of the tag as a single u8
    pub fn type_byte(&self) -> u8 {
        match self {
            &NBT::End => 0,
            &NBT::Byte(..) => 1,
            &NBT::Short(..) => 2,
            &NBT::Int(..) => 3,
            &NBT::Long(..) => 4,
            &NBT::Float(..) => 5,
            &NBT::Double(..) => 6,
            &NBT::ByteArray(..) => 7,
            &NBT::String(..) => 8,
            &NBT::List(..) => 9,
            &NBT::Compound(..) => 10,
            &NBT::IntArray(..) => 11,
        }
    }
}

/// Represents the different compression formats NBT files can be in
#[derive(Clone, PartialEq, Debug)]
pub enum Compression {
    None,
    Gzip,
    Zlib,
}
impl Compression {
    /// Returns the type of compression as an English string
    pub fn to_str(&self) -> &str {
        match self {
            &Compression::None => "None",
            &Compression::Gzip => "Gzip",
            &Compression::Zlib => "Zlib",
        }
    }
    /// Given the name of a type of compression, return the corresponding
    /// Compression enum. Returns Some(Compression) if it exists, and None if no
    /// such Compression type exists
    pub fn from_str(string: &str) -> Option<Self> {
        match string {
            "None" => Some(Compression::None),
            "Gzip" => Some(Compression::Gzip),
            "Zlib" => Some(Compression::Zlib),
            _ => None,
        }
    }
    /// Given the first byte from an NBT file, return the type of Compression
    /// used in that file. Returns Some(Compression) if the type of compression
    /// is known, and None else.
    pub fn from_first_byte(byte: u8) -> Option<Self> {
        /* On compression: To identify how an nbt file is compressed, peek
         * at the first byte in the file, with the following meanings:
         * 0x0a for no compression
         * 0x1f gzip compressed
         * 0x78 zlib compressed */
        match byte {
            0x0a => Some(Compression::None),
            0x1f => Some(Compression::Gzip),
            0x78 => Some(Compression::Zlib),
            _ => None,
        }
    }
}

/// Represents a single NBT file, that is all the NBT data, as well as a
/// compression type.
///
/// The root NBT tag will always be an NBT::Compound
#[derive(PartialEq, Debug)]
pub struct NBTFile {
    pub root: NBT,
    pub compression: Compression,
}
