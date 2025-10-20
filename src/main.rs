mod args;
mod logger;

use args::Commands;
use quilt::archive::encoder::ArchiveEncoder;
use quilt::archive::{Archive, ArchiveType};
use quilt::lzrw3a::{self, CompressAction};

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::time::Instant;

use clap::Parser;
use colored::Colorize;

const LZRW3A_CREDIT: &str = "using lzrw3a v1.0 by Ross N. Williams (15-Jul-1991)";

fn main() {
    // set virtual terminal
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).ok();

    // print header
    println!(
        "{} v{} by Brandon Gardenhire\n",
        env!("CARGO_PKG_NAME").bold(),
        env!("CARGO_PKG_VERSION")
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
            no_filenames,
        } => handle_pack(output, inputs, kub, no_filenames),
        Commands::Compress { input, output } => {
            handle_lzrw3a(CompressAction::Compress, &input, output)
        }
        Commands::Decompress { input, output } => {
            handle_lzrw3a(CompressAction::Decompress, &input, output)
        }
    } {
        // print error and exit
        logger::error(&e.to_string().bold());
        std::process::exit(1);
    }

    logger::completion(&format!("done ({:?})", start.elapsed()));
}

fn handle_pack(
    output: String,
    inputs: Vec<String>,
    kub: bool,
    no_filenames: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // ensure no filenames is only used with pak
    if no_filenames && kub {
        return Err("no_filenames flag can only be used when creating pak archives".into());
    }

    // print headers
    logger::section("Packing Archive");
    if kub {
        logger::info(LZRW3A_CREDIT);
    }

    // create encoder
    let archive_type = match kub {
        true => ArchiveType::Kub,
        false => ArchiveType::Pak,
    };

    let mut encoder = ArchiveEncoder::new(archive_type);

    // add files
    for file in inputs.iter() {
        // parse list file
        if file.starts_with('@') {
            let list_file_path = file.trim_start_matches('@');
            let list_file_dir = Path::new(list_file_path).parent().unwrap_or(Path::new("."));
            let content = fs::read_to_string(list_file_path)
                .map_err(|e| format!("failed to open file list - \"{}\"", e))?;

            for line in content.lines() {
                let line = line.trim();

                // skip empty lines or comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // resolve path relative to list file's directory
                let path = list_file_dir.join(line);
                let name = path.file_name().unwrap().to_str().unwrap();
                encoder
                    .add(&path)
                    .map_err(|e| format!("failed to pack file - \"{}\"", e))?;
                logger::completion(&format!("packed file {}", name.bold()));
            }
            continue;
        }

        let path = Path::new(file);
        let name = path.file_name().unwrap().to_str().unwrap();
        encoder
            .add(path)
            .map_err(|e| format!("failed to pack file - \"{}\"", e))?;
        logger::completion(&format!("packed file {}", name.bold()));
    }

    // create archive
    let output_path = Path::new(&output);
    encoder
        .pack(output_path, no_filenames)
        .map_err(|e| format!("failed to pack archive - \"{}\"", e))?;
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
    logger::section("Unpacking Archive");

    // open archive
    let file = File::open(input).map_err(|e| format!("failed to open archive file - \"{}\"", e))?;
    let reader = BufReader::new(file);
    let mut archive = Archive::open(reader).map_err(|e| format!("invalid archive - \"{}\"", e))?;
    logger::completion(&format!("opened archive {}", input.bold()));

    // print credit
    if archive.ty == ArchiveType::Kub {
        logger::info(LZRW3A_CREDIT);
    }

    // define output and create directory automatically
    let output_str = output.unwrap_or(String::from("."));
    let output = Path::new(&output_str);
    std::fs::create_dir_all(output)
        .map_err(|e| format!("failed to create output directory - \"{}\"", e))?;

    // define filename map and array
    let mut names: HashMap<String, u8> = HashMap::new();
    let mut names_array = Vec::<String>::new();

    // save archive contents
    for (i, e) in archive
        .entries()
        .map_err(|e| format!("failed to get archive entries - \"{}\"", e))?
        .enumerate()
    {
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
        e.unpack(&path)
            .map_err(|e| format!("failed to unpack file - \"{}\"", e))?;
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
    // print headers
    logger::section(&format!("{} LZRW3A File", action));
    logger::info(LZRW3A_CREDIT);

    // read input file
    let mut file = File::open(input)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    logger::completion(&format!("opened file {}", input.bold()));

    // process buffer
    let buffer = lzrw3a::compress(action, &buffer).ok_or("lzrw3a buffer returned null")?;
    logger::completion("processed file");

    // write output file
    let output = output.unwrap_or_else(|| format!("{}.out", input));
    let mut file = File::create(&output)?;
    file.write_all(&buffer)?;
    logger::completion(&format!("wrote file {}", output.bold()));

    Ok(())
}
