extern crate clap;
extern crate lib;

use clap::Clap;
use std::fs;

#[derive(Clap)]
#[clap(version = "1.0", author = "Jan P. <honza.spacir1@gmail.com>")]
struct Opts {
    lang: Lang,
    path: String,
    out: String,
}

#[derive(Debug)]
enum Lang {
    Rust,
    TS,
}
impl std::str::FromStr for Lang {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" => Ok(Lang::Rust),
            "ts" => Ok(Lang::TS),
            s => {
                println!("{}", s);
                Err("no match")
            }
        }
    }
}

fn main() {
    let opts = Opts::parse();
    let schema = match fs::read_to_string(&opts.path) {
        Ok(f) => f,
        Err(e) => return println!("{}", e),
    };
    println!("Compiling file {}...", opts.path);
    match match opts.lang {
        Lang::Rust => lib::compile::<lib::gen::rust::Rust>(&schema),
        Lang::TS => lib::compile::<lib::gen::ts::TypeScript>(&schema),
        //_ => return println!("not implemented"),
    } {
        Ok(generated) => {
            println!("Done.\nWriting to {}...", &opts.out);
            match fs::write(&opts.out, &generated) {
                Ok(_) => (),
                Err(e) => return println!("{}", e),
            }
            println!("Done.");
        }
        Err(e) => println!("{}", e),
    }
}
