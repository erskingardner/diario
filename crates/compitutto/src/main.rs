use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod data;
mod db;
mod html;
mod parser;
mod server;
mod types;

#[derive(Parser, Debug)]
#[command(name = "compitutto")]
#[command(about = "Parse homework calendar exports and generate a web view")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output directory for generated files
    #[arg(short, long, default_value = ".", global = true)]
    output: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the web server (default)
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Process files and generate static HTML (no server)
    Build,

    /// Process a specific file
    Parse {
        /// Path to the Excel XML file
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        // Default to serve if no command specified
        None | Some(Commands::Serve { port: 8080 }) => {
            server::serve(8080, args.output).await?;
        }
        Some(Commands::Serve { port }) => {
            server::serve(port, args.output).await?;
        }
        Some(Commands::Build) => {
            let entries = data::process_all_exports(&args.output)?;
            let html_path = args.output.join("index.html");
            html::generate_html(&entries, &html_path)?;
            println!("HTML saved: {}", html_path.display());
        }
        Some(Commands::Parse { file }) => {
            let entries = parser::parse_excel_xml(&file)?;
            println!("Found {} entries in {}", entries.len(), file.display());
            for entry in &entries {
                println!(
                    "  {} | {} | {}",
                    entry.date, entry.subject, entry.entry_type
                );
            }
        }
    }

    Ok(())
}
