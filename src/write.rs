use data::{Compression, NBT, NBTFile};
use errors::Result;

use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use flate2;
use flate2::write::{GzEncoder, ZlibEncoder};

macro_rules! compression_level {
    () => { flate2::Compression::default() };
}

/// Given an NBT file, write it as a binary NBT file to the writer
pub fn write_file<W: Write>(w: &mut W, file: &NBTFile) -> Result<()> {
    let map = match file.root {
        NBT::Compound(ref x) => x,
        _ => unreachable!(),
    };

    match file.compression {
        Compression::None => write_compound(w, &map, false)?,
        Compression::Gzip => {
            let mut w = GzEncoder::new(w, compression_level!());
            write_compound(&mut w, map, false)?;
            let _: &mut W = w.finish()?;
        },
        Compression::Zlib => {
            let mut w = ZlibEncoder::new(w, compression_level!());
            write_compound(&mut w, map, false)?;
            let _: &mut W = w.finish()?;
        },
    }

    Ok(())
}

fn write_tag<W: Write>(w: &mut W, tag: &NBT) -> Result<()> {
    match tag {
        &NBT::End => bail!("Unable to write End tag"),
        &NBT::Byte(x) => write_byte(w, x),
        &NBT::Short(x) => write_short(w, x),
        &NBT::Int(x) => write_int(w, x),
        &NBT::Long(x) => write_long(w, x),
        &NBT::Float(x) => write_float(w, x),
        &NBT::Double(x) => write_double(w, x),
        &NBT::ByteArray(ref x) => write_byte_array(w, x),
        &NBT::String(ref x) => write_string(w, x),
        &NBT::List(ref x) => write_list(w, x),
        &NBT::Compound(ref x) => write_compound(w, x, true),
        &NBT::IntArray(ref x) => write_int_array(w, x),
    }
}

fn write_byte<W: Write>(w: &mut W, val: i8) -> Result<()> {
    w.write_i8(val).map_err(|e| e.into())
}

fn write_short<W: Write>(w: &mut W, val: i16) -> Result<()> {
    w.write_i16::<BigEndian>(val).map_err(|e| e.into())
}

fn write_int<W: Write>(w: &mut W, val: i32) -> Result<()> {
    w.write_i32::<BigEndian>(val).map_err(|e| e.into())
}

fn write_long<W: Write>(w: &mut W, val: i64) -> Result<()> {
    w.write_i64::<BigEndian>(val).map_err(|e| e.into())
}

fn write_float<W: Write>(w: &mut W, val: f32) -> Result<()> {
    w.write_f32::<BigEndian>(val).map_err(|e| e.into())
}

fn write_double<W: Write>(w: &mut W, val: f64) -> Result<()> {
    w.write_f64::<BigEndian>(val).map_err(|e| e.into())
}

fn write_byte_array<W: Write>(w: &mut W, val: &Vec<i8>) -> Result<()> {
    write_int(w, val.len() as i32)?;

    for x in val {
        write_byte(w, *x)?;
    }

    Ok(())
}

fn write_string<W: Write>(w: &mut W, val: &String) -> Result<()> {
    let bytes = val.as_bytes();
    w.write_u16::<BigEndian>(bytes.len() as u16)?;
    w.write_all(bytes).map_err(|e| e.into())
}

fn write_list<W: Write>(w: &mut W, val: &Vec<NBT>) -> Result<()> {
    /* If the list has length 0, then it just defaults to type "End". */
    let tag_type = if val.len() > 0 {
        val[0].type_byte()
    } else {
        0
    };
    w.write_all(&[tag_type])?;
    write_int(w, val.len() as i32)?;

    for tag in val {
        write_tag(w, tag)?;
    }

    Ok(())
}

fn write_compound<W: Write>(w: &mut W,
                            map: &Vec<(String, NBT)>,
                            end: bool)
                            -> Result<()> {
    for &(ref key, ref tag) in map {
        w.write_all(&[tag.type_byte()])?;
        write_string(w, key)?;
        write_tag(w, &tag)?;
    }

    /* Append the End tag, but not on the implicit Compound */
    if end {
        w.write_all(&[0])?;
    }

    Ok(())
}

fn write_int_array<W: Write>(w: &mut W, val: &Vec<i32>) -> Result<()> {
    write_int(w, val.len() as i32)?;

    for x in val {
        write_int(w, *x)?;
    }

    Ok(())
}
