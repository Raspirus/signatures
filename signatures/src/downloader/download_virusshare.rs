use std::{path::Path, fs};

use log::{warn, trace, info, error};
use reqwest::StatusCode;

use crate::{threads::threadpool::ThreadPool, MAX_THREADS, MAX_RETRIES, TMP_DIR};

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
    
    for file_id in 0..=filecount {
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
    Ok(())
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