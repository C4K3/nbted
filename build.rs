use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

// All the build file currently does is try to figure out the current
// git revision, and write it to the OUT_DIR/git-revision.txt file.
fn main() {
    // Figure out the current git revision
    let git: String = match Command::new("git").arg("rev-parse").arg("HEAD").output() {
        Ok(x) => match String::from_utf8(x.stdout) {
            Ok(x) => format!(r#""{}""#, x.trim()),
            Err(e) => {
                println!("cargo:warning=build script got invalid output trying to get latest git revision: {:?}",
                                 e);
                "unknown git revision".to_string()
            }
        },
        Err(e) => {
            println!(
                "cargo:warning=build script unable to get latest git revision: {:?}",
                e
            );
            "unknown git revision".to_string()
        }
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("git-revision.txt");
    let mut f = File::create(&dest_path).unwrap();

    f.write_all(git.as_bytes()).unwrap();
}
