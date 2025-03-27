use crate::data::{NBTFile, NBT};
use crate::iter_replacer::ReplacerExt;
use crate::Result;

use byteorder::WriteBytesExt;

use std::io::Write;

/// Given an NBT file, write it to the writer in the pretty text format
pub fn write_file<W: Write>(w: &mut W, file: &NBTFile) -> Result<()> {
    write!(w, "{}", file.compression.to_str())?;
    write_tag(w, &file.root, 0, true)?;

    Ok(())
}

fn write_tag<W: Write>(w: &mut W, tag: &NBT, indent: u64, compound: bool) -> Result<()> {
    match *tag {
        NBT::End => (),
        NBT::Byte(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        }
        NBT::Short(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        }
        NBT::Int(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        }
        NBT::Long(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        }
        NBT::Float(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        }
        NBT::Double(x) => {
            if compound {
                write!(w, " ")?;
            }
            writeln!(w, "{}", x)?;
        }
        NBT::ByteArray(ref x) => {
            writeln!(w, " {}", x.len())?;
            for val in x {
                write_indent(w, indent)?;
                writeln!(w, "{}", val)?;
            }
        }
        NBT::String(ref x) => {
            if compound {
                write!(w, " ")?;
            }
            write!(w, r#"""#)?;
            /* Order is important here */
            for b in x.iter().replacer(br"\", br"\\").replacer(br#"""#, br#"\""#) {
                w.write_all(&[b])?;
            }
            writeln!(w, r#"""#)?;
        }
        NBT::List(ref x) => {
            /* If the list has length 0, then it just defaults to type "End". */
            let tag_type = if x.is_empty() {
                "End"
            } else {
                x[0].type_string()
            };
            writeln!(w, " {} {}", tag_type, x.len())?;
            for val in x {
                match val {
                    NBT::Compound(..) => (),
                    _ => write_indent(w, indent)?,
                }
                write_tag(w, val, indent + 1, false)?;
            }
        }
        NBT::Compound(ref x) => {
            if compound {
                writeln!(w)?;
            }
            for (key, val) in x {
                write_indent(w, indent)?;
                w.write_all(val.type_string().as_bytes())?;
                write!(w, r#" ""#)?;
                for x in key
                    .iter()
                    .replacer(br"\", br"\\")
                    .replacer(br#"""#, br#"\""#)
                {
                    w.write_all(&[x])?;
                }
                write!(w, r#"""#)?;
                write_tag(w, val, indent + 1, true)?;
            }

            write_indent(w, indent)?;
            writeln!(w, "End")?;
        }
        NBT::IntArray(ref x) => {
            writeln!(w, " {}", x.len())?;
            for val in x {
                write_indent(w, indent)?;
                writeln!(w, "{}", val)?;
            }
        }
        NBT::LongArray(ref x) => {
            writeln!(w, " {}", x.len())?;
            for val in x {
                write_indent(w, indent)?;
                writeln!(w, "{}", val)?;
            }
        }
    }

    Ok(())
}

fn write_indent<W: Write>(w: &mut W, indent: u64) -> Result<()> {
    for _ in 0..indent {
        /* 9 = tab character */
        w.write_u8(9)?;
    }
    Ok(())
}
