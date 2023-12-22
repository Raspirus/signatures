use std::{fs::{self, File, DirEntry}, path::Path, io::{BufReader, BufRead, Write}};

use log::{warn, trace, info, error};
use reqwest::StatusCode;

use crate::{threads::threadpool::ThreadPool, organizer::{self, database::{create_table, insert_hashes}}, MAX_THREADS, MAX_RETRIES, TMP_DIR, OUTPUT_DIR, FILE_SIZES, DATABASE};

use super::download_commons::download_file;

static URL: &str = "https://virusshare.com/hashfiles/VirusShare_";

pub fn download_all() -> std::io::Result<()> {
    let output_dir = Path::new(TMP_DIR);
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    
    let start_time = std::time::Instant::now();
    info!("Indexing webfiles...");
    let filecount = match get_file_count() {
        Ok(filecount) => filecount,
        Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Could not get maximum filecount: {err}")))
    };
    info!("Found {filecount} file(s)");

    let pool = ThreadPool::new(MAX_THREADS)?;
    
    for file_id in 0..=20 {
        pool.execute(move || {
            let download_path = output_dir.join(&format!("vs_{:0>5}.md5", file_id));
            let file_url = format!("{URL}{:0>5}.md5", file_id);
            match download_file(&download_path, &file_url, MAX_RETRIES) {
                Ok(_) => info!("Downloaded {}", download_path.display()),
                Err(err) => error!("Failed to download {file_url}: {err}"),
            };
        });
    }
    drop(pool);
    info!("Downloaded files in {}s", std::time::Instant::now().duration_since(start_time).as_secs());
    let start_time_db = std::time::Instant::now();

    let mut database = organizer::database::create_pool().expect("Failed to open database connection");
    create_table(&database).expect("Failed to create table");

    for file_id in 0..=20 {
        let reader_path = output_dir.join(&format!("vs_{:0>5}.md5", file_id));
        info!("Inserting {} into database...", reader_path.display());
        let file = match File::open(&reader_path) {
            Ok(file) => file,
            Err(err) => {
                error!("Could not open file {} for reading: {err}", reader_path.display());
                continue;
            }
        };
        let reader = BufReader::new(file);
        let mut lines = Vec::new();

        for line in reader.lines() {
            match line {
                Ok(line) => if !line.starts_with('#') { lines.push(line) },
                Err(err) => {
                    warn!("Could not read line in file {}: {err}", reader_path.display());
                    continue;
                },
            };
        }

        match insert_hashes(&mut database, &lines) {
            Ok(_) => trace!("Inserted {} hashes", lines.len()),
            Err(err) => {
                warn!("Error inserting: {err}");
            }
        }
    }

    info!("Built database in {}s", std::time::Instant::now().duration_since(start_time_db).as_secs());
    info!("Total time was {}s", std::time::Instant::now().duration_since(start_time).as_secs());
    fs::remove_dir_all(TMP_DIR)?;
    fs::remove_file(DATABASE)
}



fn get_file_count() -> Result<usize, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let mut max = 0;
    let mut max_retry = 5;
    
    // go up in 10 increments
    loop {
        let file_url = format!("{URL}{:0>5}.md5", max);
        trace!("Requesting {}", file_url);
        let response = client.head(file_url).send()?;
        match response.status() {
            StatusCode::OK => max += 10,
            StatusCode::NOT_FOUND => break,
            _ => {
                warn!("Received invalid status {}, trying again...", response.status());
                max_retry -= 1;
                if max_retry == 0 {
                    warn!("Failed 5 times, aborting; Check your network?")
                }
            }
        }
    }

    max -= 10;
    max_retry = 5;

    // go up in 1 increments from last 10th still present
    loop {
        let file_url = format!("{URL}{:0>5}.md5", max);
        trace!("Requesting {}", file_url);
        let response = client.head(file_url).send()?;
        match response.status() {
            StatusCode::OK => max += 1,
            StatusCode::NOT_FOUND => break,
            _ => {
                warn!("Received invalid status {}, trying again...", response.status());
                max_retry -= 1;
                if max_retry == 0 {
                    warn!("Failed 5 times, aborting; Check your network?")
                }
            }
        }
    }
    Ok(max - 1)
}

pub fn write_files() -> std::io::Result<()> {
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
    Ok(())
}