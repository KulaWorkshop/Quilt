use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "quilt", about)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract an archive file to a specified directory
    Unpack {
        /// Path to an archive file
        input: String,
        /// Target directory for extracted files (default: current directory)
        output: Option<String>,
        /// Generate a text file containing the list of extracted files
        #[arg(short = 's', long)]
        save_filenames: bool,
    },
    /// Create an archive file from specified input files
    Pack {
        /// Path for the output archive file
        output: String,
        /// Paths to input files. Use @FILENAME syntax to read from a file containing an input list
        inputs: Vec<String>,
        /// Create archive in KUB format (default: PAK)
        #[arg(short = 'k', long)]
        kub: bool,
        /// Create archive with no filenames
        #[arg(long)]
        no_filenames: bool,
    },
    /// Compress a file with LZRW3-A
    Compress {
        /// Path to the input file
        input: String,
        /// Path for the output file (default: INPUT_FILENAME.out)
        output: Option<String>,
    },
    /// Decompress a file with LZRW3-A
    Decompress {
        /// Path to the input file
        input: String,
        /// Path for the output file (default: INPUT_FILENAME.out)
        output: Option<String>,
    },
}
