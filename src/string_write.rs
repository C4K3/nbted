use data::{NBT, NBTFile};

use byteorder::WriteBytesExt;

use std::io;
use std::io::Write;

/// Given an NBT file, write it to the writer in the pretty text format
pub fn write_file<W: Write>(w: &mut W, file: &NBTFile) -> io::Result<()> {
    write!(w, "{}", file.compression.to_str())?;
    write_tag(w, &file.root, -1, true)?;

    Ok(())
}

fn write_tag<W: Write>(w: &mut W,
                       tag: &NBT,
                       indent: i8,
                       compound: bool)
                       -> io::Result<()> {

    match tag {
        &NBT::End => (),
        &NBT::Byte(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        },
        &NBT::Short(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        },
        &NBT::Int(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        },
        &NBT::Long(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        },
        &NBT::Float(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        },
        &NBT::Double(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        },
        &NBT::ByteArray(ref x) => {
            writeln!(w, " {}", x.len())?;
            for val in x {
                write_indent(w, indent + 1)?;
                writeln!(w, "{}", val)?;
            }
        },
        &NBT::String(ref x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w,
                     r#""{}""#,
                     /* Order is important here */
                     x.replace(r"\", r"\\").replace(r#"""#, r#"\""#))?
        },
        &NBT::List(ref x) => {
            /* If the list has length 0, then it just defaults to type "End". */
            let tag_type = if x.len() > 0 {
                x[0].type_string()
            } else {
                "End"
            };
            writeln!(w, " {} {}", tag_type, x.len())?;
            for val in x {
                match val {
                    &NBT::Compound(..) => (),
                    _ => write_indent(w, indent + 1)?,
                }
                write_tag(w, val, indent + 1, false)?;
            }
        },
        &NBT::Compound(ref x) => {
            if compound {
                writeln!(w, "")?;
            }
            for &(ref key, ref val) in x {
                write_indent(w, indent + 1)?;
                w.write_all(val.type_string().as_bytes())?;
                write!(w,
                       r#" "{}""#,
                       /* Order is important here */
                       key.replace(r"\", r"\\").replace(r#"""#, r#"\""#))?;
                write_tag(w, val, indent + 1, true)?;
            }

            write_indent(w, indent + 1)?;
            writeln!(w, "End")?;
        },
        &NBT::IntArray(ref x) => {
            writeln!(w, " {}", x.len())?;
            for val in x {
                write_indent(w, indent + 1)?;
                writeln!(w, "{}", val)?;
            }
        },
    }


    Ok(())
}

fn write_indent<W: Write>(w: &mut W, indent: i8) -> io::Result<()> {
    for _ in 0..indent {
        /* 9 = tab character */
        w.write_u8(9)?;
    }
    Ok(())
}
