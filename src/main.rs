#![warn(unused_results,
        unused_extern_crates,
        unused_import_braces,
        unused_qualifications,
        variant_size_differences,
        trivial_casts,
        trivial_numeric_casts,
        )]
#[macro_use]
extern crate failure;

use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::env;
use std::process::exit;
use std::process::Command;

use getopts::Options;

use tempdir::TempDir;

use failure::ResultExt;

type Result<T> = std::result::Result<T, failure::Error>;

pub mod data;
mod write;
mod read;
mod string_write;
mod string_read;
#[cfg(test)]
mod tests;

fn main() {
    match run_cmdline() {
        Ok(ret) => {
            exit(ret);
        },
        Err(e) => {
            eprintln!("{}", e.backtrace());

            eprintln!("Error: {}", e);

            for e in e.iter_chain().skip(1) {
                eprintln!("	caused by: {}", e);
            }

            eprintln!("For help, run with --help or read the manpage.");
            exit(1);
        },
    }
}

/// Main entrypoint for program.
///
/// Returns an integer representing the program's exit status.
fn run_cmdline() -> Result<i32> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    let _: &Options = opts.optflagopt("e", "edit", "edit a NBT file with your $EDITOR.
    If [FILE] is specified, then that file is edited in place, but specifying --input and/or --output will override the input/output.
    If no file is specified, default to read from --input and writing to --output.", "FILE");
    let _: &Options = opts.optflagopt("p", "print", "print NBT file to text format. Adding an argument to this is the same as specifying --input", "FILE");
    let _: &Options = opts.optflagopt("r", "reverse", "reverse a file in text format to NBT format. Adding an argument to this is the same as specifying --input", "FILE");
    let _: &Options = opts.optopt("i",
                "input",
                "specify the input file, defaults to stdin",
                "FILE");
    let _: &Options = opts.optopt("o",
                "output",
                "specify the output file, defaults to stdout",
                "FILE");
    let _: &Options = opts.optflag("", "man", "print the nbted man page source and exit");
    let _: &Options = opts.optflag("h", "help", "print the help menu and exit");
    let _: &Options = opts.optflag("", "version", "print program version and exit");

    let matches = opts.parse(&args[1..]).context("error parsing options")?;

    if matches.opt_present("h") {
        let brief = "Usage: nbted [options] FILE";
        print!("{}", opts.usage(&brief));
        println!("\nThe default action, taken if no action is explicitly selected, is to --edit.");
        println!("\nFor detailed usage information, read the nbted man page. If the nbted man page\
        \nwas not installed on your system, such as if you installed using `cargo install`,\
        \nthen you can use `nbted --man | nroff -man | less` to read the nbted man page.");
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

    if matches.opt_present("man") {
        print!(include_str!("../nbted.1"));
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
        bail!("You can only specify one action at a time.");
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
///
/// Returns an integer representing the program's exit status.
fn edit(input: &str, output: &str) -> Result<i32> {

    /* First we read the NBT data from the input */
    let nbt = if input == "-" {
        // let mut f = BufReader::new(io::stdin());
        let f = io::stdin();
        let mut f = f.lock();
        read::read_file(&mut f).context("Unable to parse any NBT files from stdin")?
    } else {
        let path: &Path = Path::new(input);
        let f = File::open(path)
            .context(format!("Unable to open file {}", input))?;
        let mut f = BufReader::new(f);

        read::read_file(&mut f).context(
            format_err!("Unable to parse {}, are you sure it's an NBT file?",
                        input))?
    };

    /* Then we create a temporary file and write the NBT data in text format
     * to the temporary file */
    let tmpdir = TempDir::new("nbted")
        .context("Unable to create temporary directory")?;

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
            .context("Unable to create temporary file")?;

        string_write::write_file(&mut f, &nbt)
            .context("Unable to write temporary file")?;

        f.sync_all().context("Unable to synchronize file")?;
    }

    let new_nbt = {
        let mut new_nbt = open_editor(&tmp_path);

        while let Err(e) = new_nbt {
            eprintln!("Unable to parse edited file");
            for e in e.iter_chain() {
                eprintln!("	caused by: {}", e);
            }
            eprintln!("Do you want to open the file for editing again? (y/N)");

            let mut line = String::new();
            let _: usize = io::stdin().read_line(&mut line)
                .context("Error reading from stdin. Nothing was changed")?;

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
        /* If we get an error writing to stdout, we want to just silently exit
         * with exit code 1. (It can generally be assumed that nbted will not
         * error in serializing the data, so any error here would be because of
         * writing to stdout) */
        match write::write_file(&mut f, &new_nbt) {
            Ok(()) => (),
            Err(_) => return Ok(1),
        }
    } else {
        let path: &Path = Path::new(output);
        let f = File::create(&path).context(
            format_err!("Unable to write to output NBT file {}. Nothing was changed",
                        output))?;
        let mut f = BufWriter::new(f);

        write::write_file(&mut f, &new_nbt).context(
            format_err!("Error writing NBT file {}. State of NBT file is unknown, consider restoring it from a backup.",
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
    let _: &mut Command = cmd.arg(&tmp_path.as_os_str());
    let mut cmd = cmd.spawn().context("Error opening editor")?;

    match cmd.wait().context("error executing editor")? {
        x if x.success() => (),
        _ => bail!("Editor did not exit correctly"),
    }

    /* Then we parse the text format in the temporary file into NBT */
    let mut f = File::open(&tmp_path).context(
        format_err!("Unable to read temporary file. Nothing was changed."))?;

    string_read::read_file(&mut f).map_err(|e| e.into())
}

/// When the user wants to print an NBT file to text format
fn print(input: &str, output: &str) -> Result<i32> {
    /* First we read a NBTFile from the input */
    let nbt = if input == "-" {
        let f = io::stdin();
        let mut f = f.lock();
        read::read_file(&mut f).context(
            format_err!("Unable to parse {}, are you sure it's an NBT file?",
                       input))?
    } else {
        let path: &Path = Path::new(input);
        let f = File::open(path).context(
            format_err!("Unable to open file {}", input))?;
        let mut f = BufReader::new(f);

        read::read_file(&mut f).context(
            format_err!("Unable to parse {}, are you sure it's an NBT file?",
                       input))?
    };

    /* Then we write the NBTFile to the output in text format */
    if output == "-" {
        let f = io::stdout();
        let mut f = f.lock();
        /* If we get an error writing to stdout, we want to just silently exit
         * with exit code 1. (It can generally be assumed that nbted will not
         * error in serializing the data, so any error here would be because of
         * writing to stdout) */
        match string_write::write_file(&mut f, &nbt) {
            Ok(()) => (),
            Err(_) => return Ok(1),
        }
    } else {
        let path: &Path = Path::new(output);
        let f = File::create(&path).context(
            format_err!("Unable to write to output NBT file {}. Nothing was changed.",
                       output))?;
        let mut f = BufWriter::new(f);

        string_write::write_file(&mut f, &nbt).context(
            format_err!("Error writing NBT file {}. State of NBT file is unknown, consider restoring it from a backup.",
                       output))?;
    }

    Ok(0)
}

/// When the user wants to convert a text format file into an NBT file
///
/// Returns an integer representing the program's exit status.
fn reverse(input: &str, output: &str) -> Result<i32> {
    /* First we read the input file in the text format */
    let path: &Path = Path::new(input);
    let mut f = File::open(&path).context(
        format_err!("Unable to read text file {}",
                   input))?;

    let nbt = string_read::read_file(&mut f).context(
        format_err!("Unable to parse text file {}",
        input))?;

    /* Then we write the parsed NBT to the output file in NBT format */
    if output == "-" {
        let f = io::stdout();
        let mut f = f.lock();
        /* If we get an error writing to stdout, we want to just silently exit
         * with exit code 1. (It can generally be assumed that nbted will not
         * error in serializing the data, so any error here would be because of
         * writing to stdout) */
        match write::write_file(&mut f, &nbt) {
            Ok(()) => (),
            Err(_) => return Ok(1),
        }
    } else {
        let path: &Path = Path::new(output);
        let f = File::create(&path).context(
            format_err!("Unable to write to output NBT file {}. Nothing was changed",
                       output))?;
        let mut f = BufWriter::new(f);

        write::write_file(&mut f, &nbt).context(
            format_err!("error writing to NBT FILE {}, state of NBT file is unknown, consider restoring it from a backup.",
                       output))?;
    }

    Ok(0)
}
