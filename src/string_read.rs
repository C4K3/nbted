use data::{Compression, NBT, NBTFile};
use errors::{Result, ResultExt};

use std::io::Read;

use regex::Regex;

/// Based on the idea of std::io::Cursor, letting us iterate over all the tags
/// from the pretty text format
struct Cursor {
    inner: Vec<String>,
    index: usize,
}
impl Cursor {
    fn new(tags: Vec<String>) -> Self {
        Cursor {
            inner: tags,
            index: 0,
        }
    }
    /// Gets the next item, pushing the index one forward
    fn next(&mut self) -> Result<&str> {
        match self.inner.get(self.index) {
            Some(ref x) => {
                self.index += 1;
                Ok(x)
            },
            None => bail!("Tried to read beyond Cursor"),
        }
    }
}

/// Read an NBT file from the reader, in the pretty text format
pub fn read_file<R: Read>(reader: &mut R) -> Result<NBTFile> {
    let mut buf = Vec::new();
    let _: usize = reader.read_to_end(&mut buf)?;
    let string = String::from_utf8(buf).chain_err(|| "Unable to parse string as valid UTF-8")?;

    /* We want to make a Vec<String> of all the items in the pretty text
     * format, where an item is defined as a Type, Length or other atomic
     * value.
     *
     * For almost all items this is no problem, because \S+ will match them,
     * but strings are just a slight exception, because they can contain any
     * character, including newline. */
    let re = match Regex::new(r#""(?s:((?:\\.|[^"])*)")|(\S+)"#) {
        Ok(x) => x,
        _ => unreachable!(),
    };

    let mut tags: Vec<String> = Vec::new();
    for cap in re.captures_iter(&string) {
        tags.push(match cap.get(1) {
            /* Only the first capture is a String, so we only undo the quotes
             * on the first capture, not that it would make any difference also
             * doing it on the second. Order in the replaces is important */
            Some(x) => x.as_str().replace(r#"\""#, r#"""#).replace(r"\\", r"\"),
            None => match cap.get(2) {
                Some(x) => x.as_str().to_string(),
                None => bail!("Capture did not match regex"),
            },
        });
    }

    if tags.len() < 2 {
        /* There has to be at least 2 tags: The compression, and the End tag
         * for the implicit compound */
        bail!("Invalid text file, too short");
    }

    let mut cursor = Cursor::new(tags);

    let compression = {
        let tmp = cursor.next()?;

        match Compression::from_str(tmp) {
            Some(x) => x,
            None => bail!("Unknown compression format {}", tmp),
        }
    };

    let root = read_compound(&mut cursor)?;

    Ok(NBTFile {
           root: root,
           compression: compression,
       })
}

fn read_tag(tags: &mut Cursor, tag_type: &str) -> Result<NBT> {
    match tag_type {
        "Byte" => read_byte(tags),
        "Short" => read_short(tags),
        "Int" => read_int(tags),
        "Long" => read_long(tags),
        "Float" => read_float(tags),
        "Double" => read_double(tags),
        "ByteArray" => read_byte_array(tags),
        "String" => read_string(tags),
        "List" => read_list(tags),
        "Compound" => read_compound(tags),
        "IntArray" => read_int_array(tags),
        x => bail!("Unknown tag type {}", x),
    }
}

fn read_byte(tags: &mut Cursor) -> Result<NBT> {
    let val = tags.next()?;
    let val = val.parse::<i8>().chain_err(|| format!("Invalid Byte {}", val))?;
    Ok(NBT::Byte(val))
}

fn read_short(tags: &mut Cursor) -> Result<NBT> {
    let val = tags.next()?;
    let val = val.parse::<i16>().chain_err(|| format!("Invalid Short {}", val))?;
    Ok(NBT::Short(val))
}

fn read_int(tags: &mut Cursor) -> Result<NBT> {
    let val = tags.next()?;
    let val = val.parse::<i32>().chain_err(|| format!("Invalid Int {}", val))?;
    Ok(NBT::Int(val))
}

fn read_long(tags: &mut Cursor) -> Result<NBT> {
    let val = tags.next()?;
    let val = val.parse::<i64>().chain_err(|| format!("Invalid Long {}", val))?;
    Ok(NBT::Long(val))
}

fn read_float(tags: &mut Cursor) -> Result<NBT> {
    let val = tags.next()?;
    let val = val.parse::<f32>().chain_err(|| format!("Invalid Float {}", val))?;
    Ok(NBT::Float(val))
}

fn read_double(tags: &mut Cursor) -> Result<NBT> {
    let val = tags.next()?;
    let val = val.parse::<f64>().chain_err(|| format!("Invalid Double {}", val))?;
    Ok(NBT::Double(val))
}

fn read_byte_array(tags: &mut Cursor) -> Result<NBT> {
    let len = match read_int(tags)? {
        NBT::Int(x) => x,
        _ => unreachable!(),
    };
    let mut tmp = Vec::with_capacity(len as usize);
    for _ in 0..len {
        tmp.push(match read_byte(tags)? {
                     NBT::Byte(x) => x,
                     _ => unreachable!(),
                 });
    }
    Ok(NBT::ByteArray(tmp))
}

fn read_string(tags: &mut Cursor) -> Result<NBT> {
    Ok(NBT::String(tags.next()?.to_string()))
}

fn read_list(tags: &mut Cursor) -> Result<NBT> {
    let list_type = tags.next()?.to_string();
    let len = match read_int(tags)? {
        NBT::Int(x) => x,
        _ => unreachable!(),
    };
    let mut tmp = Vec::with_capacity(len as usize);
    for _ in 0..len {
        tmp.push(read_tag(tags, &list_type)?);
    }

    Ok(NBT::List(tmp))
}

fn read_compound(tags: &mut Cursor) -> Result<NBT> {
    let mut map = Vec::new();

    loop {
        let tag_type = tags.next()?.to_string();

        /* If we get an End tag then the compound is done */
        if tag_type == "End" {
            break;
        }

        let name = tags.next()?.to_string();
        let nbt = read_tag(tags, &tag_type)?;

        map.push((name, nbt));
    }


    Ok(NBT::Compound(map))
}

fn read_int_array(tags: &mut Cursor) -> Result<NBT> {
    let len = match read_int(tags)? {
        NBT::Int(x) => x,
        _ => unreachable!(),
    };
    let mut tmp = Vec::with_capacity(len as usize);
    for _ in 0..len {
        tmp.push(match read_int(tags)? {
                     NBT::Int(x) => x,
                     _ => unreachable!(),
                 });
    }
    Ok(NBT::IntArray(tmp))
}
