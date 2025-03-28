use crate::data::{Compression, NBTFile, NBT};
use crate::Result;

use std::borrow::{Borrow, Cow};
use std::io::Read;
use std::iter::Peekable;
use std::str;

use anyhow::{anyhow, bail, Context};

/// A struct for iterating over the tokens in a given file
///
/// Where a token is considered a single value in the file,
/// such as a tag or a value. This will /almost/ only be space-separated values
/// but unfortunately strings are an exception, as strings can contain any
/// character, including newline.
struct Tokens<'a> {
    file: &'a [u8],
    a: usize,
    b: usize,
}
impl<'a> Tokens<'a> {
    fn new(file: &'a [u8]) -> Self {
        Tokens { file, a: 0, b: 0 }
    }
}
impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.file.get(self.a)?.is_whitespace() {
            self.a += 1;
        }
        /* a now matches the beginning of the next token */

        if *self.file.get(self.a)? == 0x22 {
            /* The next token is a string */
            self.a += 1; /* So we don't include the beginning " */

            self.b = self.a;

            let mut escape: bool = false;
            let mut ret: Vec<u8> = Vec::new();

            loop {
                /* 0x22 = "
                 * 0x5c = \ */
                match self.file.get(self.b)? {
                    0x22 => {
                        if escape {
                            ret.push(0x22);
                            escape = false;
                        } else {
                            self.b += 1;
                            break;
                        }
                    }
                    0x5c => {
                        if escape {
                            ret.push(0x5c);
                            escape = false;
                        } else {
                            escape = true;
                        }
                    }
                    x if escape => {
                        return Some(Err(anyhow!(
                            r#"Invalid string, tried to escape the character {} which cannot be escaped (to enter a literal \, write \\)"#,
                            x
                        )))
                    }
                    x => ret.push(*x),
                }
                self.b += 1;
            }

            let ret: String = match String::from_utf8(ret) {
                Ok(x) => x,
                Err(e) => return Some(Err(e.into())),
            };
            let ret: Cow<str> = Cow::Owned(ret);

            self.a = self.b;
            Some(Ok(ret))
        } else {
            /* The next token is not a string */
            self.b = self.a;

            while let Some(x) = self.file.get(self.b) {
                if x.is_whitespace() {
                    break;
                } else {
                    self.b += 1;
                }
            }

            let ret = match str::from_utf8(self.file.get(self.a..self.b)?) {
                Ok(x) => x,
                Err(e) => return Some(Err(e.into())),
            };

            self.a = self.b;
            Some(Ok(Cow::Borrowed(ret)))
        }
    }
}

trait IsWhitespace {
    fn is_whitespace(&self) -> bool;
}
impl IsWhitespace for u8 {
    fn is_whitespace(&self) -> bool {
        match *self {
            0x09 => true, /* Tab */
            0x0a => true, /* Newline */
            0x0b => true, /* Vertical tab */
            0x0c => true, /* Form feed */
            0x0d => true, /* Carriage return */
            0x20 => true, /* Space */
            _ => false,
        }
    }
}

/// Read an NBT file from the reader, in the pretty text format
pub fn read_file<R: Read>(reader: &mut R) -> Result<NBTFile> {
    let mut buf = Vec::new();
    let _: usize = reader.read_to_end(&mut buf)?;

    let mut tokens = Tokens::new(&buf).peekable();

    let mut compression = None;

    while let Some(Ok(token)) = tokens.next_if(|token| match token.as_deref() {
        // This closure is used to check if the next token is a header (return True) or the
        // beginning of the contents of the NBT file (return False) The root tag must always be a
        // Compound in an NBT file, so if we see Compound we know the headers are finished. nbted
        // has also previously supported parsing empty files (containing only a single End) so
        // support that as well.
        Err(_) => false,
        Ok("Compound") => false,
        Ok("End") => false,
        Ok(_) => true,
    }) {
        match Compression::from_str(token.borrow()) {
            Some(x) => {
                if compression.is_some() {
                    bail!("Found multiple compression settings");
                }

                compression = Some(x);
            }
            None => bail!("Unknown header '{}'", token),
        }
    }

    // Default to no compression if not specified
    let compression = compression.unwrap_or(Compression::None);

    if tokens.peek().is_none() {
        bail!("NBT file in text format does not contain any tags at all");
    }

    let root = read_compound(&mut tokens)?;

    Ok(NBTFile { root, compression })
}

fn read_tag(tokens: &mut Peekable<Tokens>, tag_type: &str) -> Result<NBT> {
    match tag_type {
        "Byte" => read_byte(tokens),
        "Short" => read_short(tokens),
        "Int" => read_int(tokens),
        "Long" => read_long(tokens),
        "Float" => read_float(tokens),
        "Double" => read_double(tokens),
        "ByteArray" => read_byte_array(tokens),
        "String" => read_string(tokens),
        "List" => read_list(tokens),
        "Compound" => read_compound(tokens),
        "IntArray" => read_int_array(tokens),
        "LongArray" => read_long_array(tokens),
        x => bail!("Unknown tag type {}", x),
    }
}

fn read_byte(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let val = match tokens.next() {
        Some(x) => x?,
        None => bail!("EOF when trying to read a byte"),
    };
    let val = val
        .parse::<i8>()
        .with_context(|| format!("Invalid Byte {}", val))?;
    Ok(NBT::Byte(val))
}

fn read_short(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let val = match tokens.next() {
        Some(x) => x?,
        None => bail!("EOF when trying to read a short"),
    };
    let val = val
        .parse::<i16>()
        .context(format!("Invalid Short {}", val))?;
    Ok(NBT::Short(val))
}

fn read_int(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let val = match tokens.next() {
        Some(x) => x?,
        None => bail!("EOF when trying to read an int"),
    };
    let val = val.parse::<i32>().context(format!("Invalid Int {}", val))?;
    Ok(NBT::Int(val))
}

fn read_long(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let val = match tokens.next() {
        Some(x) => x?,
        None => bail!("EOF when trying to read a long"),
    };
    let val = val
        .parse::<i64>()
        .context(format!("Invalid Long {}", val))?;
    Ok(NBT::Long(val))
}

fn read_float(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let val = match tokens.next() {
        Some(x) => x?,
        None => bail!("EOF when trying to read a float"),
    };
    let val = val
        .parse::<f32>()
        .context(format!("Invalid Float {}", val))?;
    Ok(NBT::Float(val))
}

fn read_double(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let val = match tokens.next() {
        Some(x) => x?,
        None => bail!("EOF when trying to read a double"),
    };
    let val = val
        .parse::<f64>()
        .context(format!("Invalid Double {}", val))?;
    Ok(NBT::Double(val))
}

fn read_byte_array(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let len = match read_int(tokens)? {
        NBT::Int(x) => x,
        _ => unreachable!(),
    };
    let mut tmp = Vec::with_capacity(len as usize);
    for _ in 0..len {
        tmp.push(match read_byte(tokens)? {
            NBT::Byte(x) => x,
            _ => unreachable!(),
        });
    }
    Ok(NBT::ByteArray(tmp))
}

fn read_string(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let val = match tokens.next() {
        Some(x) => x?,
        None => bail!("EOF when trying to read a string"),
    };
    Ok(NBT::String(val.into_owned().into_bytes()))
}

fn read_list(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let list_type = match tokens.next() {
        Some(x) => x?,
        None => bail!("EOF when trying to read a list type"),
    };
    let len = match read_int(tokens)? {
        NBT::Int(x) => x,
        _ => unreachable!(),
    };
    let mut tmp = Vec::with_capacity(len as usize);
    for _ in 0..len {
        tmp.push(read_tag(tokens, &list_type)?);
    }

    Ok(NBT::List(tmp))
}

fn read_compound(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let mut map = Vec::new();

    loop {
        let tag_type = match tokens.next() {
            Some(x) => x?,
            None => bail!("EOF when trying to read the next item in a compound"),
        };

        /* If we get an End tag then the compound is done */
        if &tag_type == "End" {
            break;
        }

        let name = match tokens.next() {
            Some(x) => x?,
            None => bail!(
                "EOF when trying to read the name of a {} tag in a compound",
                tag_type
            ),
        };
        let nbt = read_tag(tokens, &tag_type)?;

        map.push((name.into_owned().into_bytes(), nbt));
    }

    Ok(NBT::Compound(map))
}

fn read_int_array(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let len = match read_int(tokens)? {
        NBT::Int(x) => x,
        _ => unreachable!(),
    };
    let mut tmp = Vec::with_capacity(len as usize);
    for _ in 0..len {
        tmp.push(match read_int(tokens)? {
            NBT::Int(x) => x,
            _ => unreachable!(),
        });
    }
    Ok(NBT::IntArray(tmp))
}

fn read_long_array(tokens: &mut Peekable<Tokens>) -> Result<NBT> {
    let len = match read_int(tokens)? {
        NBT::Int(x) => x,
        _ => unreachable!(),
    };
    let mut tmp = Vec::with_capacity(len as usize);
    for _ in 0..len {
        tmp.push(match read_long(tokens)? {
            NBT::Long(x) => x,
            _ => unreachable!(),
        });
    }
    Ok(NBT::LongArray(tmp))
}
