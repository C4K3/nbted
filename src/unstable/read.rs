use crate::data::{Compression, NBTFile, NBT};
use crate::Result;

use std::io::{self, BufRead, Read};

use byteorder::{BigEndian, ReadBytesExt};

use flate2::read::{GzDecoder, ZlibDecoder};

/// Read an NBT file from the given reader
pub fn read_file<R: BufRead>(mut reader: &mut R) -> Result<NBTFile> {
    /* Peek into the first byte of the reader, which is used to determine the
     * compression */
    let peek = match reader.fill_buf()? {
        x if !x.is_empty() => x[0],
        _ => bail!("Error peaking first byte in read::read_file, file was EOF"),
    };

    let compression = match Compression::from_first_byte(peek) {
        Some(x) => x,
        None => bail!("Unknown compression format where first byte is {}", peek),
    };

    let root = match compression {
        Compression::None => read_compound(&mut reader)?,
        Compression::Gzip => read_compound(&mut GzDecoder::new(reader))?,
        Compression::Zlib => read_compound(&mut ZlibDecoder::new(reader))?,
    };

    Ok(NBTFile { root, compression })
}

/// Reads an NBT compound. I.e. assumes that the first byte from the Reader is
/// the byte that determines the NBT type of the first value INSIDE whatever
/// compound we're in.
///
/// This will always return an NBT::Compound, never any other type of NBT.
fn read_compound<R: Read>(reader: &mut R) -> Result<NBT> {
    let mut map = Vec::new();

    loop {
        let mut buf: [u8; 1] = [0];

        /* If unable to read anything, then we're done */
        match reader.read_exact(&mut buf) {
            Ok(()) => (),
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                return Err(e.into());
            }
        }

        /* If we've got a TAG_end now, then the compound list is done */
        if buf[0] == 0x0 {
            break;
        }

        map.push((
            match read_string(reader)? {
                NBT::String(val) => val,
                _ => unreachable!(),
            },
            match buf[0] {
                0x01 => read_byte(reader)?,
                0x02 => read_short(reader)?,
                0x03 => read_int(reader)?,
                0x04 => read_long(reader)?,
                0x05 => read_float(reader)?,
                0x06 => read_double(reader)?,
                0x07 => read_byte_array(reader)?,
                0x08 => read_string(reader)?,
                0x09 => read_list(reader)?,
                0x0a => read_compound(reader)?,
                0x0b => read_int_array(reader)?,
                0x0c => read_long_array(reader)?,
                x => {
                    bail!("Got unknown type id {:x} trying to read NBT compound", x);
                }
            },
        ));
    }

    Ok(NBT::Compound(map))
}

fn read_byte<R: Read>(reader: &mut R) -> Result<NBT> {
    Ok(NBT::Byte(reader.read_i8()?))
}

fn read_short<R: Read>(reader: &mut R) -> Result<NBT> {
    Ok(NBT::Short(reader.read_i16::<BigEndian>()?))
}

fn read_int<R: Read>(reader: &mut R) -> Result<NBT> {
    Ok(NBT::Int(reader.read_i32::<BigEndian>()?))
}

fn read_long<R: Read>(reader: &mut R) -> Result<NBT> {
    Ok(NBT::Long(reader.read_i64::<BigEndian>()?))
}

fn read_float<R: Read>(reader: &mut R) -> Result<NBT> {
    Ok(NBT::Float(reader.read_f32::<BigEndian>()?))
}

fn read_double<R: Read>(reader: &mut R) -> Result<NBT> {
    Ok(NBT::Double(reader.read_f64::<BigEndian>()?))
}

fn read_byte_array<R: Read>(reader: &mut R) -> Result<NBT> {
    let length = match read_int(reader)? {
        NBT::Int(val) => val as usize,
        _ => unreachable!(),
    };

    let mut ret: Vec<i8> = Vec::with_capacity(length);

    for _ in 0..length {
        ret.push(match read_byte(reader)? {
            NBT::Byte(val) => val,
            _ => unreachable!(),
        });
    }

    Ok(NBT::ByteArray(ret))
}

fn read_string<R: Read>(reader: &mut R) -> Result<NBT> {
    /* Apparently the length of a string is given unsigned unlike everything
     * else in NBT */
    let length = reader.read_u16::<BigEndian>()?;

    let mut buf = Vec::with_capacity(length as usize);
    let tmp = reader.take(length as u64).read_to_end(&mut buf)?;
    if tmp != length as usize {
        bail!("Error reading string length");
    }

    Ok(NBT::String(buf))
}

fn read_list<R: Read>(reader: &mut R) -> Result<NBT> {
    let mut type_id: [u8; 1] = [0];
    reader.read_exact(&mut type_id)?;

    let length = match read_int(reader)? {
        NBT::Int(val) => val as usize,
        _ => unreachable!(),
    };

    let mut ret: Vec<NBT> = Vec::new();
    for _ in 0..length {
        ret.push(match type_id[0] {
            0x0 => NBT::End,
            0x1 => read_byte(reader)?,
            0x2 => read_short(reader)?,
            0x3 => read_int(reader)?,
            0x4 => read_long(reader)?,
            0x5 => read_float(reader)?,
            0x6 => read_double(reader)?,
            0x7 => read_byte_array(reader)?,
            0x8 => read_string(reader)?,
            0x9 => read_list(reader)?,
            0xa => read_compound(reader)?,
            0xb => read_int_array(reader)?,
            0xc => read_long_array(reader)?,
            x => bail!("Got unknown type id {:x} trying to read NBT list", x),
        });
    }

    Ok(NBT::List(ret))
}

fn read_int_array<R: Read>(reader: &mut R) -> Result<NBT> {
    let length = match read_int(reader)? {
        NBT::Int(val) => val as usize,
        _ => unreachable!(),
    };

    let mut ret: Vec<i32> = Vec::new();

    for _ in 0..length {
        ret.push(match read_int(reader)? {
            NBT::Int(val) => val,
            _ => unreachable!(),
        });
    }

    Ok(NBT::IntArray(ret))
}

fn read_long_array<R: Read>(reader: &mut R) -> Result<NBT> {
    let length = match read_int(reader)? {
        NBT::Int(val) => val as usize,
        _ => unreachable!(),
    };

    let mut ret: Vec<i64> = Vec::new();

    for _ in 0..length {
        ret.push(match read_long(reader)? {
            NBT::Long(val) => val,
            _ => unreachable!(),
        });
    }

    Ok(NBT::LongArray(ret))
}
