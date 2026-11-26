mod article;
mod code;
mod content;
mod figure;
mod formula;
mod highlight;
mod links;
mod output;
mod pages;
mod photo;
mod programs;
mod scrub;
mod serve;
mod site;
mod style;
mod tree;
mod typeset;

use std::path::PathBuf;
use std::{env, fs};

use anyhow::{Context, Result, ensure};
use clap::Parser;

use crate::output::File;
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
    /// Take the location and camera details out of photos, and look for them in other files
    Scrub { files: Vec<PathBuf> },
}

fn main() -> Result<()> {
    let command = Command::parse();
    let here = env::current_dir()?;
    let root = here.ancestors().find(|folder| folder.join("site.toml").exists());
    env::set_current_dir(root.context("there is no site.toml here")?)?;
    // Typst goes deep.
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
        Command::Scrub { files } => {
            for path in files.iter().map(|file| here.join(file)) {
                if let File::Made(clean) = scrub::scrub(&path)?
                    && clean != fs::read(&path)?
                {
                    fs::write(&path, clean)?;
                    println!("scrubbed {}", path.display());
                }
            }
        }
    }
    Ok(())
}
