use std::{path::Path, fs::{File, self}, io::Write};

use reqwest::StatusCode;

/// downloads a file from file_url and save it to output_name. it expects the path to the output name to already exist
pub fn download_file(output_name: &Path, file_url: &str) -> std::io::Result<()>{
    output_name.exists().then(|| fs::remove_file(output_name));
    let mut file = File::create(output_name)?;
    let client = reqwest::blocking::Client::new();
    let response = match client.get(file_url).send() {
        Ok(response) => response,
        Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))
    };

    match response.status() {
        StatusCode::OK => match response.text() {
            Ok(data) => file.write_all(data.as_bytes())?,
            Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())),
        },
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Got response code {}", response.status())))
    }
    Ok(())
}