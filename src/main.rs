mod args;
mod logger;

use args::Commands;
use clap::Parser;
use colored::Colorize;
use quilt::archive::encoder::ArchiveEncoder;
use quilt::archive::{Archive, ArchiveType};
use quilt::lzrw3a::{self, CompressAction};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::Path;
use std::time::Instant;

fn main() {
    // set virtual terminal
    colored::control::set_virtual_terminal(true).unwrap();

    // print header
    println!(
        "{} {} by Brandon Gardenhire\n",
        env!("CARGO_PKG_NAME").bold(),
        format!("v{}", env!("CARGO_PKG_VERSION"))
    );

    // parse arguments
    let arguments = args::Arguments::parse();

    // process file
    let start = Instant::now();
    if let Err(e) = match arguments.command {
        Commands::Unpack {
            input,
            output,
            save_filenames,
        } => handle_unpack(&input, output, save_filenames),
        Commands::Pack {
            output,
            inputs,
            kub,
        } => handle_pack(output, inputs, kub),
        Commands::Compress { input, output } => {
            handle_lzrw3a(CompressAction::Compress, &input, output)
        }
        Commands::Decompress { input, output } => {
            handle_lzrw3a(CompressAction::Decompress, &input, output)
        }
    } {
        logger::error(&e.to_string().bold());
        std::process::exit(1);
    }

    logger::completion(&format!("done ({:?})", start.elapsed()));
}

fn handle_pack(
    output: String,
    inputs: Vec<String>,
    kub: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // print credit
    if kub {
        logger::info("using lzrw3a v1.0 by Ross N. Williams (15-Jul-1990)");
    }

    // add files
    let mut encoder = ArchiveEncoder::new(if kub {
        ArchiveType::Kub
    } else {
        ArchiveType::Pak
    });
    for file in inputs.iter() {
        if file.starts_with('@') {
            let content = fs::read_to_string(file.trim_start_matches('@'))?;
            for line in content.lines() {
                let path = Path::new(line);
                let name = path.file_name().unwrap().to_str().unwrap();
                encoder.add(path)?;
                logger::completion(&format!("packed file {}", name.bold()));
            }
            continue;
        }

        let path = Path::new(file);
        let name = path.file_name().unwrap().to_str().unwrap();
        encoder.add(path)?;
        logger::completion(&format!("packed file {}", name.bold()));
    }

    // create archive
    let output_path = Path::new(&output);
    encoder.pack(output_path)?;
    logger::completion(&format!(
        "wrote archive {}",
        output_path.file_name().unwrap().to_str().unwrap().bold()
    ));

    Ok(())
}

fn handle_unpack(
    input: &str,
    output: Option<String>,
    save_filenames: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // print header
    logger::section(&format!("Unpacking Archive"));

    // define output
    let output_str = output.unwrap_or(String::from("."));
    let output = Path::new(&output_str);
    std::fs::create_dir_all(output)?;

    // open archive
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    logger::completion(&format!("opened file {}", input));
    let mut archive = Archive::open(reader)?;

    // print credit
    if archive.ty == ArchiveType::Kub {
        logger::info("using lzrw3a v1.0 by Ross N. Williams (15-Jul-1990)");
    }

    // define filename map
    let mut names: HashMap<String, u8> = HashMap::new();
    let mut names_array = Vec::<String>::new();

    // save archive contents
    for (i, e) in archive.entries()?.enumerate() {
        // add name to map
        let mut name = e.name.clone().unwrap_or(format!("FILE {}", i + 1));
        let entry = names.entry(name.clone());
        entry.and_modify(|v| *v += 1).or_insert(2);

        // check for duplicate filenames
        if let Some(v) = names.get(&name)
            && v > &2
        {
            logger::info(&format!("duplicate filename {}", name.bold()));
            name = format!("{}-{}", name, v - 1);
        }

        // unpack file and add filename to log array
        let path = output.join(&name);
        e.unpack(&path)?;
        logger::completion(&format!("wrote file {}", name.bold()));
        names_array.push(name);
    }

    // save filenames to log file
    if save_filenames {
        // create log file
        let input = Path::new(&input);
        let archive_name = input.file_name().unwrap().to_str().unwrap();
        let log_name = format!("{}.txt", archive_name);
        let mut file = File::create(output.join(&log_name))?;

        // write filenames to log
        for name in &names_array {
            writeln!(file, "{}", name)?;
        }

        logger::completion(&format!("wrote filenames to {}", log_name.bold()));
    }

    Ok(())
}

fn handle_lzrw3a(
    action: lzrw3a::CompressAction,
    input: &str,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // define output
    let output = output.unwrap_or_else(|| format!("{}.out", input));

    // print headers
    logger::section(&format!("{} LZRW3A File", action));
    logger::info("using lzrw3a v1.0 by Ross N. Williams (15-Jul-1990)");

    // read input file
    let mut file = File::open(input)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    logger::completion(&format!("opened file {}", input.bold()));

    // process buffer
    let result = lzrw3a::compress(action, &buffer);
    if result.is_none() {
        return Err("algorithm failed".into());
    }

    let buffer = result.unwrap();
    logger::completion("processed file");

    // write output file
    io::stdout().flush().unwrap();
    let mut file = File::create(&output)?;
    file.write_all(&buffer)?;

    logger::completion(&format!("wrote file {}", output.bold()));

    Ok(())
}
