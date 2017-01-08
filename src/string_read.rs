use ::data::{NBT, NBTFile, Compression};

use std::io;
use std::io::{BufRead, BufReader, Read};
use std::ops::Deref;

use regex::Regex;

/** Read an NBT file from the reader, in the pretty text format */
pub fn read_file<R: Read>(reader: &mut R) -> io::Result<NBTFile> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\s*(\S+)$").unwrap();
    }

    let mut r = BufReader::new(reader);
    let mut tmp = String::new();
    let _: usize = r.read_line(&mut tmp)?;
    let tmp = tmp.trim_right();

    let captures = match RE.captures(&tmp) {
        Some(x) => x,
        None => {
            return io_error!("First line did not match regex: {}", &tmp);
        },
    };

    let compression: Compression = match captures.get(1) {
        Some(x) => match Compression::from_str(x.as_str()) {
            Some(x) => x,
            None => return io_error!("Invalid compression: {}", x.as_str()),
        },
        None => unreachable!(),
    };

    Ok(NBTFile {
        root: read_compound(&mut r)?,
        compression: compression,
    })
}

fn read_compound<R: BufRead>(r: &mut R) -> io::Result<NBT> {
    let mut map: Vec<(String, NBT)> = Vec::new();

    loop {
        lazy_static! {
            /* The capture groups here represent
             * 1: Type of tag in compound
             * 2: Name of tag in compound
             * 3: Value of tag if String, else None.
             * 4: Value of tag if atomic (except string).
             *    Length of array if ByteArray/IntArray.
             *    Type of list if lift. None if Compound.
             * 5: If list: The length of list, else None. */
            static ref RE: Regex = Regex::new(
                r#"^\s*(\S+) ''(.*?)''(?: ''(.*)''| (\S+))?(?: (\S+))?$"#)
                .unwrap();
            static ref RE_END: Regex = Regex::new(r"^\s*End$").unwrap();
        }

        let mut line = String::new();
        r.read_line(&mut line)?;
        let line = line.trim_right().to_string();

        if RE_END.is_match(&line) {
            break;
        }

        let captures = match RE.captures(&line) {
            Some(x) => x,
            None => return io_error!(
                "Line did not match regex while parsing compound: {}", &line),
        };

        let tag_type: String = match captures.get(1) {
            Some(x) => x.as_str().to_string(),
            None => unreachable!(),
        };

        let name: String = match captures.get(2) {
            Some(x) => x.as_str().replace(r#"\'"#, r#"'"#),
            None => unreachable!(),
        };

        /* Whether we should expect a value to be in this line, i.e.
         * atomic NBT values will have their value on the same line, and thus
         * this will be true, but compounds and lists/arrays will have them
         * on the next lines, and thus this will be false. Here we also check
         * that the tag_type is valid */
        let expect_value: bool = match tag_type.deref() {
            "End" => false,
            "Byte" => true,
            "Short" => true,
            "Int" => true,
            "Long" => true,
            "Float" => true,
            "Double" => true,
            "ByteArray" => false,
            "String" => true,
            "List" => false,
            "Compound" => false,
            "IntArray" => false,
            _ => return io_error!("Invalid tag type {} in line {}",
                                  &tag_type, &line),
        };

        let nbt: NBT = if expect_value {
            let value = match captures.get(3) {
                Some(x) => x.as_str().to_string(),
                None => match captures.get(4) {
                    Some(x) => x.as_str().to_string(),
                    None => return io_error!(
                        "Expected an atomic value but did not get one on line {}",
                        &line),
                },
            };

            match tag_type.deref() {
                "Byte" => NBT::Byte(match value.parse::<i8>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid i8 {} on line {}",
                                               value, &line),
                }),
                "Short" => NBT::Short(match value.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid i16 {} on line {}",
                                               value, &line),
                }),
                "Int" => NBT::Int(match value.parse::<i32>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid i32 {} on line {}",
                                               value, &line),
                }),
                "Long" => NBT::Long(match value.parse::<i64>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid i64 {} on line {}",
                                               value, &line),
                }),
                "Float" => NBT::Float(match value.parse::<f32>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid f32 {} on line {}",
                                               value, &line),
                }),
                "Double" => NBT::Double(match value.parse::<f64>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid f64 {} on line {}",
                                               value, &line),
                }),
                "String" => {
                    let string: String = value.replace(r#"\'"#, r#"'"#);
                    NBT::String(string)
                },
                _ => unreachable!(),
            }

        } else {
            let cap4 = match captures.get(4) {
                Some(x) => Some(x.as_str()),
                None => None,
            };
            let cap5 = match captures.get(5) {
                Some(x) => Some(x.as_str()),
                None => None,
            };

            match tag_type.deref() {
                "ByteArray" => read_byte_array(r, cap4)?,
                "List" => read_list(r, cap4, cap5)?,
                "Compound" => read_compound(r)?,
                "IntArray" => read_int_array(r, cap4)?,
                _ => unreachable!(),
            }
        };

        map.push((name, nbt));
    }

    Ok(NBT::Compound(map))
}

fn read_byte_array<R: BufRead>(r: &mut R, len: Option<&str>)
-> io::Result<NBT> {
    let len = match len {
        Some(ref x) => match x.parse::<usize>() {
            Ok(x) => x,
            Err(_) => return io_error!("Invalid usize {}", x),
        },
        None => return io_error!(
            "Expected a value but did not get one while parsing byte array"),
    };

    let mut ret: Vec<i8> = Vec::with_capacity(len);

    for _ in 0..len {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^\s*(\S+)$").unwrap();
        }

        let mut line = String::new();
        r.read_line(&mut line)?;
        let line = line.trim_right().to_string();

        let captures = match RE.captures(&line) {
            Some(x) => x,
            None => return io_error!(
                "Line did not match regex while parsing byte array: {}", &line),
        };

        let number = match captures.get(1) {
            Some(x) => x.as_str(),
            None => unreachable!(),
        };

        let number = match number.parse::<i8>() {
            Ok(x) => x,
            Err(_) => return io_error!("Invalid i8 {} in line {}",
                                       number, &line),
        };

        ret.push(number);
    }

    Ok(NBT::ByteArray(ret))
}

fn read_list<R: BufRead>(r: &mut R, tag_type: Option<&str>, len: Option<&str>)
-> io::Result<NBT> {
    let len = match len {
        Some(ref x) => match x.parse::<usize>() {
            Ok(x) => x,
            Err(_) => return io_error!("Invalid usize {}", x),
        },
        None => return io_error!(
            "Expected a length but did not get one while parsing list"),
    };

    let tag_type: String = match tag_type {
        Some(ref x) => x.to_string(),
        None => return io_error!(
            "Expected a tag type but did not get one while parsing list"),
    };

    let mut ret: Vec<NBT> = Vec::with_capacity(len);

    for _ in 0..len {
        if tag_type == "Compound" {
            ret.push(read_compound(r)?);
            continue;
        }

        let mut line = String::new();
        r.read_line(&mut line)?;
        let line = line.trim_right().to_string();

        /* Whether we should expect a value to be in this line, i.e.
         * atomic NBT values will have their value on the same line, and thus
         * this will be true, but compounds and lists/arrays will have them
         * on the next lines, and thus this will be false. Here we also check
         * that the tag_type is valid */
        let expect_value: bool = match tag_type.deref() {
            "End" => false,
            "Byte" => true,
            "Short" => true,
            "Int" => true,
            "Long" => true,
            "Float" => true,
            "Double" => true,
            "ByteArray" => false,
            "String" => true,
            "List" => false,
            "Compound" => false,
            "IntArray" => false,
            _ => return io_error!("Invalid tag type {}", &tag_type),
        };

        let nbt: NBT = if expect_value {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^\s*(.*)$").unwrap();
            }

            let captures = match RE.captures(&line) {
                Some(x) => x,
                None => {
                    return io_error!("Line in list did not match regex");
                },
            };

            let value = match captures.get(1) {
                Some(x) => x.as_str().to_string(),
                None => unreachable!(),
            };

            match tag_type.deref() {
                "Byte" => NBT::Byte(match value.parse::<i8>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid i8 {} in line {}",
                                               value, &line),
                }),
                "Short" => NBT::Short(match value.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid i16 {} in line {}",
                                               value, &line),
                }),
                "Int" => NBT::Int(match value.parse::<i32>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid i32 {} in line {}",
                                               value, &line),
                }),
                "Long" => NBT::Long(match value.parse::<i64>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid i64 {} in line {}",
                                               value, &line),
                }),
                "Float" => NBT::Float(match value.parse::<f32>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid f32 {} in line {}",
                                               value, &line),
                }),
                "Double" => NBT::Double(match value.parse::<f64>() {
                    Ok(x) => x,
                    Err(_) => return io_error!("Invalid f64 {} in line {}",
                                               value, &line),
                }),
                "String" => NBT::String(value.replace(r#"\'"#, r#"'"#)),
                _ => unreachable!(),
            }

        } else {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^\s*(\S+) ?(\S+)?$")
                    .unwrap();
            }

            let captures = match RE.captures(&line) {
                Some(x) => x,
                None => {
                    return io_error!("Line in list did not match regex");
                },
            };

            let cap1 = match captures.get(1) {
                Some(x) => Some(x.as_str()),
                None => None,
            };
            let cap2 = match captures.get(2) {
                Some(x) => Some(x.as_str()),
                None => None,
            };

            match tag_type.deref() {
                "ByteArray" => read_byte_array(r, cap1)?,
                "List" => read_list(r, cap1, cap2)?,
                "IntArray" => read_int_array(r, cap1)?,
                _ => unreachable!(),
            }
        };

        ret.push(nbt);

    }

    Ok(NBT::List(ret))
}

fn read_int_array<R: BufRead>(r: &mut R, len: Option<&str>) -> io::Result<NBT> {
    let len = match len {
        Some(ref x) => match x.parse::<usize>() {
            Ok(x) => x,
            Err(_) => return io_error!("Invalid usize {}", x),
        },
        None => return io_error!(
            "Expected a length but did not get one while parsing list"),
    };

    let mut ret: Vec<i32> = Vec::with_capacity(len);

    for _ in 0..len {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^\s*(\S+)$").unwrap();
        }

        let mut line = String::new();
        r.read_line(&mut line)?;
        let line = line.trim_right().to_string();

        let captures = match RE.captures(&line) {
            Some(x) => x,
            None => return io_error!(
                "Line did not match regex while parsing byte array"),
        };

        let number = match captures.get(1) {
            Some(x) => x.as_str(),
            None => unreachable!(),
        };

        let number = match number.parse::<i32>() {
            Ok(x) => x,
            Err(_) => return io_error!("Invalid i32 {} in line {}",
                                       number, &line),
        };

        ret.push(number);
    }

    Ok(NBT::IntArray(ret))
}

