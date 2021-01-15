extern crate clap;
extern crate lib;

use anyhow::Result;
use clap::Clap;
use std::fs;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = "1.0", author = "Jan P. <honza.spacir1@gmail.com>")]
struct Opts {
    lang: Lang,
    path: String,
    out: String,
}

#[derive(Clone, Copy, Debug)]
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

fn extension(lang: Lang) -> &'static str {
    match lang {
        Lang::Rust => "rs",
        Lang::TS => "ts",
    }
}

fn visit_files(dir: &str, cb: &mut dyn FnMut(&fs::DirEntry) -> Result<()>) -> Result<()> {
    if fs::metadata(dir)?.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_files(path.to_str().unwrap(), cb)?;
            } else if path.extension().is_some() && path.extension().unwrap() == "pkt" {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Schema {
    path: PathBuf,
    generated: String,
}

fn run_one(path: String, lang: Lang) -> Result<Schema> {
    Ok(Schema {
        path: PathBuf::from(path.clone()),
        generated: match lang {
            Lang::Rust => lib::compile::<lib::gen::rust::Rust>(&fs::read_to_string(path)?)?,
            Lang::TS => lib::compile::<lib::gen::ts::TypeScript>(&fs::read_to_string(path)?)?,
            //_ => return println!("not implemented"),
        },
    })
}

fn run_all(dir: String, lang: Lang) -> Result<Vec<Schema>> {
    let mut out = Vec::new();
    visit_files(&dir, &mut |entry| {
        // damn, this is ugly...
        out.push(run_one(
            entry
                .path()
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid path"))?
                .into(),
            lang,
        )?);
        Ok(())
    })?;
    Ok(out)
}

fn save_one(schema: Schema, lang: Lang, inp: &str, outp: &str) -> Result<()> {
    let path = schema.path.clone();

    let full_out_path = format!(
        "{outp}/{parent}",
        outp = outp,
        parent = path
            .strip_prefix(inp)?
            .parent()
            .unwrap()
            .to_str()
            .map_or_else(String::new, |v| if v.is_empty() {
                String::new()
            } else {
                format!("{}/", v)
            })
    )
    .chars()
    // also transform path separators
    .map(|c| if c == '\\' { '/' } else { c })
    .collect::<String>();

    fs::create_dir_all(&full_out_path)?;

    let out = format!(
        "{dir}{filename}.{ext}",
        dir = full_out_path,
        filename = schema.path.file_stem().unwrap().to_str().unwrap(),
        ext = extension(lang)
    );
    println!("Writing to {}", out);
    fs::write(out, schema.generated)?;

    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    if let Ok(meta) = fs::metadata(&opts.path) {
        if meta.is_dir() {
            let results = run_all(opts.path.clone(), opts.lang)?;
            for result in results {
                save_one(result, opts.lang, &opts.path, &opts.out)?;
            }
        } else {
            let result = run_one(opts.path.clone(), opts.lang)?;
            save_one(result, opts.lang, &opts.path, &opts.out)?;
        }
    }

    Ok(())
}
