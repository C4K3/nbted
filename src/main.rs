extern crate byteorder;
extern crate regex;
extern crate flate2;
extern crate getopts;
extern crate tempdir;

use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::env;
use std::process::exit;
use std::process::Command;
use std::error::Error;

use getopts::Options;

use tempdir::TempDir;

/** Create an InvalidData io::Error with the description being a
 * formatted string */
macro_rules! io_error {
    ($fmtstr:tt) => { io_error!($fmtstr,) };
    ($fmtstr:tt, $( $args:expr ),* ) => {
        Err(::std::io::Error::new(::std::io::ErrorKind::InvalidData,
                                  format!($fmtstr, $( $args ),* )));
    };
}

/** Alternative to println! that prints to stderr instead of stdout */
macro_rules! printerrln {
    ($fmtstr:tt) => { printerrln!($fmtstr,) };
    ($fmtstr:tt, $( $args:expr ),* ) => {
        writeln!(&mut io::stderr(), $fmtstr, $( $args ),* ).silent_unwrap();
    }
}

pub mod data;
mod write;
mod read;
mod string_write;
mod string_read;
#[cfg(test)]
mod tests;

/// Trait used in place of unwrap for printing to stdout/stderr, since if that
/// errors the program should simply exit with no further output
trait SilentUnwrap<T> {
    fn silent_unwrap(self) -> T;
}
impl<T> SilentUnwrap<T> for io::Result<T> {
    fn silent_unwrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(_) => std::process::exit(1),
        }
    }
}

fn main() {
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

    let matches = match opts.parse(&args[1..]) {
        Ok(x) => x,
        Err(e) => {
            error(&format!("Error parsing options: {:?}", e.description()))
        },
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    if matches.opt_present("version") {
        println!("{} {} {}",
                 env!("CARGO_PKG_NAME"),
                 env!("CARGO_PKG_VERSION"),
                 /* See build.rs for the git-revision.txt file */
                 include!(concat!(env!("OUT_DIR"), "/git-revision.txt")));
        println!("{}", env!("CARGO_PKG_HOMEPAGE"));
        return;
    }

    /* Figure out the input file, by trying to read the arguments for all of
     * --input, --edit, --print and --reverse, prioritizing --input over the
     * other arguments, if none of the arguments are specified but there is a
     * free argument, use that, else we finally default to - (stdin) */
    let input = match matches.opt_str("input") {
        Some(x) => x,
        None => {
            match matches.opts_str(&["edit".to_string(),
                                     "print".to_string(),
                                     "reverse".to_string()]) {

                Some(x) => x,
                None if matches.free.len() > 0 => matches.free[0].clone(),
                None => "-".to_string(),
            }
        },
    };

    /* Analogous to the input */
    let output = match matches.opt_str("output") {
        Some(x) => x,
        None => {
            match matches.opt_str("edit") {
                Some(x) => x,
                None if matches.free.len() > 0 => matches.free[0].clone(),
                None => "-".to_string(),
            }
        },
    };

    if matches.opt_present("print") {
        print(&input, &output);
        return;
    }

    if matches.opt_present("reverse") {
        reverse(&input, &output);
        return;
    }

    /* Default to --edit */
    edit(&input, &output);
}

/** When the user wants to edit a specific file in place */
fn edit(input: &str, output: &str) {

    /* First we read the NBT data from the input */
    let nbt = if input == "-" {
        // let mut f = BufReader::new(io::stdin());
        let f = io::stdin();
        let mut f = f.lock();
        match read::read_file(&mut f) {
            Ok(x) => x,
            Err(_) => {
                error(&format!("Unable to parse {}, are you sure it's an NBT file?",
                              input))
            },
        }
    } else {
        let path: &Path = Path::new(input);
        let f = match File::open(path) {
            Ok(f) => f,
            Err(_) => error(&format!("Unable to open file {}", input)),
        };
        let mut f = BufReader::new(f);

        match read::read_file(&mut f) {
            Ok(x) => x,
            Err(_) => {
                error(&format!("Unable to parse {}, are you sure it's an NBT file?",
                              input))
            },
        }
    };

    /* Then we create a temporary file and write the NBT data in text format
     * to the temporary file */
    let tmpdir = match TempDir::new("nbted") {
        Ok(x) => x,
        Err(e) => {
            error(&format!("Unable to create temporary directory: {:?}",
                          e.description()))
        },
    };

    let tmp = match Path::new(input).file_name() {
        Some(x) => {
            let mut x = x.to_os_string();
            x.push(".txt");
            x
        },
        None => error(&format!("Error reading file name")),
    };
    let tmp_path = tmpdir.path().join(tmp);

    {
        let mut f = match File::create(&tmp_path) {
            Ok(x) => x,
            Err(e) => {
                error(&format!("Unable to create temporary file: {:?}",
                              e.description()))
            },
        };

        match string_write::write_file(&mut f, &nbt) {
            Ok(()) => (),
            Err(e) => {
                error(&format!("Unable to write temporary file: {:?}",
                              e.description()))
            },
        };

        match f.sync_all() {
            Ok(()) => (),
            Err(e) => {
                error(&format!("Unable to synchronize file: {:?}",
                              e.description()))
            },
        }
    }

    let new_nbt = {
        let mut new_nbt = open_editor(&tmp_path);

        while let Err(e) = new_nbt {
            printerrln!("Unable to parse edited file: {}.\n\
            Do you want to open the file for editing again? (y/N)",
                        e.description());

            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(_) => (),
                Err(e) => {
                    error(&format!("Error reading from stdin: {}\n\
                                   Nothing was changed.",
                                  e.description()))
                },
            }

            if line.trim() == "y" {
                new_nbt = open_editor(&tmp_path);
            } else {
                printerrln!("Exiting ... File is unchanged.");
                std::process::exit(1);
            }
        }

        new_nbt.expect("new_nbt was Error")
    };

    if nbt == new_nbt {
        printerrln!("No changes, will do nothing.");
        return;
    }

    /* And finally we write the edited nbt (new_nbt) into the output file */
    if output == "-" {
        let f = io::stdout();
        let mut f = f.lock();
        write::write_file(&mut f, &new_nbt).silent_unwrap();
    } else {
        let path: &Path = Path::new(output);
        let f = match File::create(&path) {
            Ok(x) => x,
            Err(e) => {
                error(&format!("Unable to write to output NBT file {}: {:?}.\n\
            Nothing was changed",
                              output,
                              e.description()))
            },
        };
        let mut f = BufWriter::new(f);

        match write::write_file(&mut f, &new_nbt) {
            Ok(()) => (),
            Err(e) => {
                error(&format!("Error writing NBT file {}: {:?}.\n\
            State of NBT file is unknown, consider restoring it from a backup.",
                              output,
                              e.description()))
            },
        }
    }

    printerrln!("File edited successfully.");
}

/// Open the user's $EDITOR on the temporary file, wait until the editor is
/// closed again, read the temporary file and attempt to parse it into NBT,
/// returning the result.
fn open_editor(tmp_path: &Path) -> io::Result<data::NBTFile> {
    let editor = match env::var("EDITOR") {
        Ok(x) => x,
        Err(_) => {
            match env::var("VISUAL") {
                Ok(x) => x,
                Err(_) => error("Unable to find $EDITOR"),
            }
        },
    };

    let mut cmd = Command::new(editor);
    cmd.arg(&tmp_path.as_os_str());
    let mut cmd = match cmd.spawn() {
        Ok(x) => x,
        Err(e) => {
            error(&format!("Error opening editor: {:?}", e.description()))
        },
    };

    match cmd.wait() {
        Ok(x) if x.success() => (),
        Err(e) => {
            error(&format!("Error executing editor: {:?}", e.description()))
        },
        _ => error("Editor did not exit correctly"),
    }

    /* Then we parse the text format in the temporary file into NBT */
    let mut f = match File::open(&tmp_path) {
        Ok(x) => x,
        Err(e) => {
            error(&format!("Unable to read temporary file: {:?}.\n\
        Nothing was changed",
                          e.description()))
        },
    };

    string_read::read_file(&mut f)
}

/** When the user wants to print an NBT file to text format */
fn print(input: &str, output: &str) {
    /* First we read a NBTFile from the input */
    let nbt = if input == "-" {
        let f = io::stdin();
        let mut f = f.lock();
        match read::read_file(&mut f) {
            Ok(x) => x,
            Err(_) => {
                error(&format!("Unable to parse {}, are you sure it's an NBT file?",
                              input))
            },
        }
    } else {
        let path: &Path = Path::new(input);
        let f = match File::open(path) {
            Ok(f) => f,
            Err(_) => error(&format!("Unable to open file {}", input)),
        };
        let mut f = BufReader::new(f);

        match read::read_file(&mut f) {
            Ok(x) => x,
            Err(_) => {
                error(&format!("Unable to parse {}, are you sure it's an NBT file?",
                              input))
            },
        }
    };

    /* Then we write the NBTFile to the output in text format */
    if output == "-" {
        let f = io::stdout();
        let mut f = f.lock();
        string_write::write_file(&mut f, &nbt).silent_unwrap();
    } else {
        let path: &Path = Path::new(output);
        let f = match File::create(&path) {
            Ok(x) => x,
            Err(e) => {
                error(&format!("Unable to write to output NBT file {}: {:?}.\n\
            Nothing was changed",
                              output,
                              e.description()))
            },
        };
        let mut f = BufWriter::new(f);

        match string_write::write_file(&mut f, &nbt) {
            Ok(()) => (),
            Err(e) => {
                error(&format!("Error writing NBT file {}: {:?}.\n\
            State of NBT file is unknown, consider restoring it from a backup.",
                              output,
                              e.description()))
            },
        }
    }
}

/** When the user wants to convert a text format file into an NBT file */
fn reverse(input: &str, output: &str) {
    /* First we read the input file in the text format */
    let path: &Path = Path::new(input);
    let mut f = match File::open(&path) {
        Ok(x) => x,
        Err(e) => {
            error(&format!("Unable to read text file {}: {:?}",
                          input,
                          e.description()))
        },
    };

    let nbt = match string_read::read_file(&mut f) {
        Ok(x) => x,
        Err(e) => {
            error(&format!("Unable to parse text file {}: {:?}",
                          input,
                          e.description()))
        },
    };

    /* Then we write the parsed NBT to the output file in NBT format */
    if output == "-" {
        let f = io::stdout();
        let mut f = f.lock();
        write::write_file(&mut f, &nbt).silent_unwrap();
    } else {
        let path: &Path = Path::new(output);
        let f = match File::create(&path) {
            Ok(x) => x,
            Err(e) => {
                error(&format!("Unable to write to output NBT file {}: {:?}.\n\
            Nothing was changed",
                              output,
                              e.description()))
            },
        };
        let mut f = BufWriter::new(f);

        match write::write_file(&mut f, &nbt) {
            Ok(()) => (),
            Err(e) => {
                error(&format!("Error writing NBT file {}: {:?}.\n\
            State of NBT file is unknown, consider restoring it from a backup.",
                              output,
                              e.description()))
            },
        }
    }
}

/** Print the given message and exit with status code 1 */
fn error(message: &str) -> ! {
    printerrln!("Error: {}", message);
    printerrln!("Run with --help for help, or read the manpage.");
    exit(1);
}

fn print_usage(opts: Options) {
    let brief = "Usage: nbted [options] FILE";
    print!("{}", opts.usage(&brief));
    println!("\nFor detailed usage information, read the manpage.");
}
