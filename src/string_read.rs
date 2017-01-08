use ::data::{NBT, NBTFile, Compression};

use std::io;
use std::io::Read;

use regex::Regex;

/** Based on the idea of std::io::Cursor, letting us iterate over all the tags
 * from the pretty text format */
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
    /** Gets the next item, pushing the index one forward */
    fn next(&mut self) -> io::Result<&str> {
        match self.inner.get(self.index) {
            Some(ref x) => {
                self.index += 1;
                Ok(x)
            },
            None => io_error!("Tried to read beyond Cursor"),
        }
    }
}

/** Read an NBT file from the reader, in the pretty text format */
pub fn read_file<R: Read>(reader: &mut R) -> io::Result<NBTFile> {
    let mut buf = Vec::new();
    let _: usize = reader.read_to_end(&mut buf)?;
    let string = match String::from_utf8(buf) {
        Ok(x) => x,
        Err(_) => return io_error!("Unable to parse string as valid UTF-8"),
    };

    /* We want to make a Vec<String> of all the items in the pretty text format,
     * where an item is defined as a Type, Length or other atomic value.
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
                None => return io_error!("Capture did not match regex"),
            },
        });
    };

    if tags.len() < 2 {
        /* There has to be at least 2 tags: The compression, and the End tag
         * for the implicit compound */
        return io_error!("Invalid text file, too short");
    }

    let mut cursor = Cursor::new(tags);

    let compression = {
        let tmp = cursor.next()?;

        match Compression::from_str(tmp) {
            Some(x) => x,
            None => return io_error!("Unknown compression format {}", tmp),
        }
    };

    let root = read_compound(&mut cursor)?;

    Ok(NBTFile {
        root: root,
        compression: compression,
    })
}

fn read_tag(tags: &mut Cursor, tag_type: &str) -> io::Result<NBT> {
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
        _ => io_error!("Unknown tag type {}", tag_type),
    }
}

fn read_byte(tags: &mut Cursor) -> io::Result<NBT> {
    let val = tags.next()?;
    match val.parse::<i8>() {
        Ok(x) => Ok(NBT::Byte(x)),
        Err(_) => io_error!("Invalid Byte {}", val),
    }
}

fn read_short(tags: &mut Cursor) -> io::Result<NBT> {
    let val = tags.next()?;
    match val.parse::<i16>() {
        Ok(x) => Ok(NBT::Short(x)),
        Err(_) => io_error!("Invalid Short {}", val),
    }
}

fn read_int(tags: &mut Cursor) -> io::Result<NBT> {
    let val = tags.next()?;
    match val.parse::<i32>() {
        Ok(x) => Ok(NBT::Int(x)),
        Err(_) => io_error!("Invalid Int {}", val),
    }
}

fn read_long(tags: &mut Cursor) -> io::Result<NBT> {
    let val = tags.next()?;
    match val.parse::<i64>() {
        Ok(x) => Ok(NBT::Long(x)),
        Err(_) => io_error!("Invalid Long {}", val),
    }
}

fn read_float(tags: &mut Cursor) -> io::Result<NBT> {
    let val = tags.next()?;
    match val.parse::<f32>() {
        Ok(x) => Ok(NBT::Float(x)),
        Err(_) => io_error!("Invalid Float {}", val),
    }
}

fn read_double(tags: &mut Cursor) -> io::Result<NBT> {
    let val = tags.next()?;
    match val.parse::<f64>() {
        Ok(x) => Ok(NBT::Double(x)),
        Err(_) => io_error!("Invalid Double {}", val),
    }
}

fn read_byte_array(tags: &mut Cursor) -> io::Result<NBT> {
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

fn read_string(tags: &mut Cursor) -> io::Result<NBT> {
    Ok(NBT::String(tags.next()?.to_string()))
}

fn read_list(tags: &mut Cursor) -> io::Result<NBT> {
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

fn read_compound(tags: &mut Cursor) -> io::Result<NBT> {
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

fn read_int_array(tags: &mut Cursor) -> io::Result<NBT> {
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

