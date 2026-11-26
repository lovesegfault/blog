mod article;
mod code;
mod content;
mod figure;
mod formula;
mod highlight;
mod links;
mod math;
mod output;
mod pages;
mod photo;
mod programs;
mod serve;
mod site;
mod style;
mod tree;
mod typeset;

use std::path::PathBuf;
use std::{env, fs};

use anyhow::{Context, Result, ensure};
use clap::Parser;

use crate::site::Tools;

#[derive(Parser)]
#[command(about)]
enum Command {
    /// Start a draft
    New { title: String },
    /// Preview the site, drafts included, as it is written
    Serve {
        #[arg(long, default_value = "127.0.0.1:1313")]
        at: String,
    },
    /// Turn a draft into a post, dated now
    Publish { draft: String },
    /// Build the site
    Build {
        #[arg(long)]
        drafts: bool,
        /// Where to put it, if not in `public`
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Take the location and camera details out of photos
    Scrub { photos: Vec<PathBuf> },
}

fn main() -> Result<()> {
    let command = Command::parse();
    let here = env::current_dir()?;
    let root = here.ancestors().find(|folder| folder.join("site.toml").exists());
    env::set_current_dir(root.context("there is no site.toml here")?)?;
    // Typst and MathJax both go deep.
    rayon::ThreadPoolBuilder::new().stack_size(16 << 20).build_global()?;

    match command {
        Command::New { title } => println!("{}", content::draft(&title)?.display()),
        Command::Serve { at } => serve::serve(&Tools::new()?, &at)?,
        Command::Publish { draft } => println!("{}", content::publish(&draft)?.display()),
        Command::Build { drafts, out } => {
            let tools = Tools::new()?;
            let site = site::build(&tools, &programs::compile(&tools.cache)?, drafts)?;
            site.problems.iter().for_each(|problem| eprintln!("{problem}"));
            ensure!(site.problems.is_empty(), "nothing was written");
            output::write(&site.files, &out.map_or("public".into(), |out| here.join(out)))?;
        }
        Command::Scrub { photos } => {
            for path in photos.iter().map(|photo| here.join(photo)) {
                let photo = output::read(&path)?;
                let clean = photo::scrub(&photo).with_context(|| path.display().to_string())?;
                if clean != photo {
                    fs::write(&path, clean)?;
                    println!("scrubbed {}", path.display());
                }
            }
        }
    }
    Ok(())
}
