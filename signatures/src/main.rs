use std::path::Path;

mod downloader;
mod threads;
fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    downloader::download_virusshare::download_all(Path::new("tmp"))?;
    Ok(())
}
