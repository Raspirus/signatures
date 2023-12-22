use std::{path::Path, fs::{File, self, DirEntry}, io::{Write, BufReader, BufRead}};

use log::{warn, info, error, debug};
use reqwest::StatusCode;

use crate::{OUTPUT_DIR, organizer::{self, database::{create_table, insert_hashes}}, FILE_SIZES, TMP_DIR, MAX_FILE_COMBINES};

/// downloads a file from file_url and save it to output_name. it expects the path to the output name to already exist
pub fn download_file(output_name: &Path, file_url: &str, max_retries: usize) -> std::io::Result<()>{
    output_name.exists().then(|| fs::remove_file(output_name));
    let mut file = File::create(output_name)?;
    let client = reqwest::blocking::Client::new();
    
    for current_retry in 0..=max_retries {
        let response = match client.get(file_url).send() {
            Ok(response) => response,
            Err(err) => { warn!("Failed to download {file_url} on try {current_retry}: {err}"); continue }
        };
    
        match response.status() {
            StatusCode::OK => match response.text() {
                Ok(data) => return Ok(file.write_all(data.as_bytes())?),
                Err(err) => warn!("Failed to download {file_url} on try {current_retry}: {err}"),
            },
            _ => warn!("Failed to download {file_url} on try {current_retry}; Statuscode was {}", response.status())
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Could not download file"))
}

/// writes the database hashes to output files
pub fn write_files() -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    let output_dir = Path::new(OUTPUT_DIR);
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;

    let files: Vec<DirEntry> = fs::read_dir(output_dir)?.filter_map(Result::ok).collect();
    let mut max = 0;
    for file in files {
        let out = file.file_name().to_str().unwrap_or_default().parse::<usize>().unwrap_or_default();
        if out > max { max = out }
    }
    if max > 0 { max += 1 }

    let connection = organizer::database::create_pool().expect("Failed to get connection");
    let mut current_frame = 0;
    let mut current_file = max;
    loop {
        let bottom = current_frame * FILE_SIZES;
        let top = bottom + FILE_SIZES;
        let hashes = organizer::database::get_hashes(&connection, bottom, top).expect("Failed to fetch hashes from db");
        if hashes.is_empty() { break }
        let mut file = File::create(Path::new(&format!("{OUTPUT_DIR}/{:0>5}", current_file)))?;
        info!("Writing to {OUTPUT_DIR}/{:0>5}", current_file);
        for hash in &hashes {
            writeln!(file, "{}", hash)?;
        }
        current_file += 1;
        current_frame += 1;
    }
    info!("Writing output files took {}s", std::time::Instant::now().duration_since(start_time).as_secs());
    Ok(())
}

pub fn insert_files() -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    let entries: Vec<DirEntry> = fs::read_dir(Path::new(TMP_DIR))?.filter_map(Result::ok).collect();
    let output_dir = Path::new(TMP_DIR);

    let mut database = organizer::database::create_pool().expect("Failed to open database connection");
    create_table(&database).expect("Failed to create table");
    
    for chunk_id in 0..=(entries.len() / MAX_FILE_COMBINES) {
        let start = chunk_id * MAX_FILE_COMBINES;
        let end = std::cmp::min((chunk_id + 1) * MAX_FILE_COMBINES, entries.len());

        let mut lines: Vec<String> = Vec::new();
        for file_id in start..end {
            let reader_path = output_dir.join(entries.get(file_id).expect("Failed to get file from entries").file_name());
            debug!("Adding {} to batch", reader_path.display());
            let file = match File::open(&reader_path) {
                Ok(file) => file,
                Err(err) => {
                    error!("Could not open file {} for reading: {err}", reader_path.display());
                    continue;
                }
            };
            let reader = BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(line) => if !line.starts_with('#') { lines.push(line) },
                    Err(err) => {
                        warn!("Could not read line in file {}: {err}", reader_path.display());
                        continue;
                    },
                };
            }         
        }
        
        info!("Inserting chunk {}/{} containing {} hashes into database...", chunk_id, (entries.len() / MAX_FILE_COMBINES), lines.len());
        match insert_hashes(&mut database, &lines) {
            Ok(_) => {},
            Err(err) => {
                warn!("Error inserting: {err}");
            }
        }
    }
    info!("Building database took {}s", std::time::Instant::now().duration_since(start_time).as_secs());
    Ok(())
}
