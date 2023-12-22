use std::{path::Path, fs::{File, self}, io::Write};

use log::warn;
use reqwest::StatusCode;

/// downloads a file from file_url and save it to output_name. it expects the path to the output name to already exist
pub fn download_file(output_name: &Path, file_url: &str, max_retries: usize) -> std::io::Result<()>{
    output_name.exists().then(|| fs::remove_file(output_name));
    let mut file = File::create(output_name)?;
    let client = reqwest::blocking::Client::new();
    
    for current_retry in 0..=max_retries {
        let response = match client.get(file_url).send() {
            Ok(response) => response,
            Err(err) => { warn!("Faild to download {file_url} on try {current_retry}: {err}"); continue }
        };
    
        match response.status() {
            StatusCode::OK => match response.text() {
                Ok(data) => return Ok(file.write_all(data.as_bytes())?),
                Err(err) => warn!("Faild to download {file_url} on try {current_retry}: {err}"),
            },
            _ => warn!("Faild to download {file_url} on try {current_retry}; Statuscode was {}", response.status())
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Could not download file"))
}