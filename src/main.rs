extern crate byteorder;
extern crate regex;
extern crate flate2;
extern crate getopts;
extern crate tempdir;
#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::env;
use std::process::exit;
use std::process::Command;

use getopts::Options;

use tempdir::TempDir;

pub mod data;
mod write;
mod read;
mod string_write;
mod string_read;
#[cfg(test)]
mod tests;

mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            FromUtf8(::std::string::FromUtf8Error);
        }
    }
}
use errors::*;

fn main() {
    match run_cmdline() {
        Ok(ret) => {
            exit(ret);
        },
        Err(ref e) => {
            if let Some(backtrace) = e.backtrace() {
                eprintln!("{:?}", backtrace);
            }

            eprintln!("Error: {}", e);

            for e in e.iter().skip(1) {
                eprintln!("	caused by: {}", e);
            }

            eprintln!("For help, run with --help or read the manpage.");
            exit(1);
        },
    }
}

fn run_cmdline() -> Result<i32> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflagopt("e", "edit", "edit a NBT file with your $EDITOR.
    If [FILE] is specified, then that file is edited in place, but specifying --input and/or --output will override the input/output.
    If no file is specified, default to read from --input and writing to --output.", "FILE");
    opts.optflagopt("p", "print", "print NBT file to text format. Adding an argument to this is the same as specifying --input", "FILE");
    opts.optflagopt("r", "reverse", "reverse a file in text format to NBT format. Adding an argument to this is the same as specifying --input", "FILE");
    opts.optopt("i",
                "input",
                "specify the input file, defaults to stdin",
                "FILE");
    opts.optopt("o",
                "output",
                "specify the output file, defaults to stdout",
                "FILE");
    opts.optflag("h", "help", "print the help menu");
    opts.optflag("", "version", "print program version");

    let matches = opts.parse(&args[1..]).chain_err(|| "error parsing options")?;

    if matches.opt_present("h") {
        let brief = "Usage: nbted [options] FILE";
        print!("{}", opts.usage(&brief));
        println!("\nThe default action, taken if no action is explicitly selected, is to --edit.");
        println!("\nFor detailed usage information, read the manpage.");
        return Ok(0);
    }

    if matches.opt_present("version") {
        println!("{} {} {}",
                 env!("CARGO_PKG_NAME"),
                 env!("CARGO_PKG_VERSION"),
                 /* See build.rs for the git-revision.txt file */
                 include!(concat!(env!("OUT_DIR"), "/git-revision.txt")));
        println!("https://github.com/C4K3/nbted");
        return Ok(0);
    }

    let is_print: bool = matches.opt_present("print");
    let is_reverse: bool = matches.opt_present("reverse");
    let is_edit: bool = if matches.opt_present("edit") {
        true
    } else {
        /* If edit is not explicitly defined, it is the default action and is
         * selected if no other action is specified */
        (!(is_reverse || is_print))
    };

    /* Hopefully this is a simpler way of ensuring that only one action can be
     * taken than having a long logical expression */
    let mut action_count = 0;
    if is_print {
        action_count += 1;
    }
    if is_reverse {
        action_count += 1;
    }
    if is_edit {
        action_count += 1;
    }
    if action_count > 1 {
        bail!("You can only specify one action a time.");
    }

    /* Figure out the input file, by trying to read the arguments for all of
     * --input, --edit, --print and --reverse, prioritizing --input over the
     * other arguments, if none of the arguments are specified but there is a
     * free argument, use that, else we finally default to - (stdin) */
    let input = if let Some(x) = matches.opt_str("input") {
        x
    } else if let Some(x) = matches.opt_str("edit") {
        x
    } else if let Some(x) = matches.opt_str("print") {
        x
    } else if let Some(x) = matches.opt_str("reverse") {
        x
    } else if matches.free.len() == 1 {
        matches.free[0].clone()
    } else {
        /* stdin */
        "-".to_string()
    };

    let output = if let Some(x) = matches.opt_str("output") {
        x
    } else if let Some(x) = matches.opt_str("edit") {
        x
    } else if is_edit && matches.free.len() == 1 {
        /* Only want to default to the free argument if we're editing
         * (DO NOT WRITE BACK TO THE READ FILE UNLESS EDITING!) */
        matches.free[0].clone()
    } else {
        /* stdout */
        "-".to_string()
    };

    if matches.free.len() > 1 {
        bail!("nbted was given multiple arguments, but only supports editing one file at a time.");
    }

    if is_print {
        return print(&input, &output);
    } else if is_reverse {
        return reverse(&input, &output);
    } else if is_edit {
        return edit(&input, &output);
    } else {
        bail!("Internal error: No action selected. (Please report this.)");
    }
}

/// When the user wants to edit a specific file in place
fn edit(input: &str, output: &str) -> Result<i32> {

    /* First we read the NBT data from the input */
    let nbt = if input == "-" {
        // let mut f = BufReader::new(io::stdin());
        let f = io::stdin();
        let mut f = f.lock();
        read::read_file(&mut f).chain_err(|| "Unable to parse any NBT files from stdin")?
    } else {
        let path: &Path = Path::new(input);
        let f = File::open(path)
            .chain_err(|| format!("Unable to open file {}", input))?;
        let mut f = BufReader::new(f);

        read::read_file(&mut f).chain_err(
            || format!("Unable to parse {}, are you sure it's an NBT file?",
                       input))?
    };

    /* Then we create a temporary file and write the NBT data in text format
     * to the temporary file */
    let tmpdir = TempDir::new("nbted")
        .chain_err(|| "Unable to create temporary directory")?;

    let tmp = match Path::new(input).file_name() {
        Some(x) => {
            let mut x = x.to_os_string();
            x.push(".txt");
            x
        },
        None => bail!("Error reading file name"),
    };
    let tmp_path = tmpdir.path().join(tmp);

    {
        let mut f = File::create(&tmp_path)
            .chain_err(|| "Unable to create temporary file")?;

        string_write::write_file(&mut f, &nbt)
            .chain_err(|| "Unable to write temporary file")?;

        f.sync_all().chain_err(|| "Unable to synchronize file")?;
    }

    let new_nbt = {
        let mut new_nbt = open_editor(&tmp_path);

        while let Err(e) = new_nbt {
            eprintln!("Unable to parse edited file");
            for e in e.iter() {
                eprintln!("	caused by: {}", e);
            }
            eprintln!("Do you want to open the file for editing again? (y/N)");

            let mut line = String::new();
            io::stdin().read_line(&mut line)
                .chain_err(|| "Error reading from stdin. Nothing was changed")?;

            if line.trim() == "y" {
                new_nbt = open_editor(&tmp_path);
            } else {
                eprintln!("Exiting ... File is unchanged.");
                return Ok(0);
            }
        }

        new_nbt.expect("new_nbt was Error")
    };

    if nbt == new_nbt {
        eprintln!("No changes, will do nothing.");
        return Ok(0);
    }

    /* And finally we write the edited nbt (new_nbt) into the output file */
    if output == "-" {
        let f = io::stdout();
        let mut f = f.lock();
        match write::write_file(&mut f, &new_nbt) {
            Ok(()) => (),
            Err(_) => return Ok(1),
        }
    } else {
        let path: &Path = Path::new(output);
        let f = File::create(&path).chain_err(
            || format!("Unable to write to output NBT file {}. Nothing was changed",
                       output))?;
        let mut f = BufWriter::new(f);

        write::write_file(&mut f, &new_nbt).chain_err(
            || format!("Error writing NBT file {}. State of NBT file is unknown, consider restoring it from a backup.",
                       output))?;
    }

    eprintln!("File edited successfully.");
    Ok(0)
}

/// Open the user's $EDITOR on the temporary file, wait until the editor is
/// closed again, read the temporary file and attempt to parse it into NBT,
/// returning the result.
fn open_editor(tmp_path: &Path) -> Result<data::NBTFile> {
    let editor = match env::var("EDITOR") {
        Ok(x) => x,
        Err(_) => {
            match env::var("VISUAL") {
                Ok(x) => x,
                Err(_) => bail!("Unable to find $EDITOR"),
            }
        },
    };

    let mut cmd = Command::new(editor);
    cmd.arg(&tmp_path.as_os_str());
    let mut cmd = cmd.spawn().chain_err(|| "Error opening editor")?;

    match cmd.wait() {
        Ok(x) if x.success() => (),
        Ok(_) => bail!("Editor did not exit correctly"),
        Err(e) => {
            return Err(Error::with_chain(e, "Error executing editor"));
        },
    }

    /* Then we parse the text format in the temporary file into NBT */
    let mut f = File::open(&tmp_path).chain_err(
        || "Unable to read temporary file. Nothing was changed.")?;

    string_read::read_file(&mut f).map_err(|e| e.into())
}

/// When the user wants to print an NBT file to text format
fn print(input: &str, output: &str) -> Result<i32> {
    /* First we read a NBTFile from the input */
    let nbt = if input == "-" {
        let f = io::stdin();
        let mut f = f.lock();
        read::read_file(&mut f).chain_err(
            || format!("Unable to parse {}, are you sure it's an NBT file?",
                       input))?
    } else {
        let path: &Path = Path::new(input);
        let f = File::open(path).chain_err(
            || format!("Unable to open file {}", input))?;
        let mut f = BufReader::new(f);

        read::read_file(&mut f).chain_err(
            || format!("Unable to parse {}, are you sure it's an NBT file?",
                       input))?
    };

    /* Then we write the NBTFile to the output in text format */
    if output == "-" {
        let f = io::stdout();
        let mut f = f.lock();
        match string_write::write_file(&mut f, &nbt) {
            Ok(()) => (),
            Err(_) => return Ok(1),
        }
    } else {
        let path: &Path = Path::new(output);
        let f = File::create(&path).chain_err(
            || format!("Unable to write to output NBT file {}. Nothing was changed.",
                       output))?;
        let mut f = BufWriter::new(f);

        string_write::write_file(&mut f, &nbt).chain_err(
            || format!("Error writing NBT file {}. State of NBT file is unknown, consider restoring it from a backup.",
                       output))?;
    }

    Ok(0)
}

/// When the user wants to convert a text format file into an NBT file
fn reverse(input: &str, output: &str) -> Result<i32> {
    /* First we read the input file in the text format */
    let path: &Path = Path::new(input);
    let mut f = File::open(&path).chain_err(
        || format!("Unable to read text file {}",
                   input))?;

    let nbt = string_read::read_file(&mut f).chain_err(
        || format!("Unable to parse text file {}",
        input))?;

    /* Then we write the parsed NBT to the output file in NBT format */
    if output == "-" {
        let f = io::stdout();
        let mut f = f.lock();
        match write::write_file(&mut f, &nbt) {
            Ok(()) => (),
            Err(_) => return Ok(1),
        }
    } else {
        let path: &Path = Path::new(output);
        let f = File::create(&path).chain_err(
            || format!("Unable to write to output NBT file {}. Nothing was changed",
                       output))?;
        let mut f = BufWriter::new(f);

        write::write_file(&mut f, &nbt).chain_err(
            || format!("error writing to NBT FILE {}, state of NBT file is unknown, consider restoring it from a backup.",
                       output))?;
    }

    Ok(0)
}
