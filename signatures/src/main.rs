use log::info;

mod downloader;
mod organizer;
mod threads;

static TMP_DIR: &str = "tmp";
static MAX_THREADS: usize = 10;
static MAX_RETRIES: usize = 5;

static DATABASE: &str = "hashes_db";
static TABLE_NAME: &str = "hashes";
static MAX_FILE_COMBINES: usize = 4;

static FILE_SIZES: usize = 1_000_000;
static OUTPUT_DIR: &str = "hashes";

fn main() -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    pretty_env_logger::init();
    //downloader::download_virusshare::download_all()?;
    downloader::download_virusshare::build_db()?;
    downloader::download_virusshare::write_files()?;
    info!("Total time was {}s", std::time::Instant::now().duration_since(start_time).as_secs());
    Ok(())
}
