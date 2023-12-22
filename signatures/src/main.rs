mod downloader;
mod organizer;
mod threads;

static TMP_DIR: &str = "tmp";
static MAX_THREADS: usize = 10;
static MAX_RETRIES: usize = 5;

static DATABASE: &str = "hashes_db";
static TABLE_NAME: &str = "hashes";

static FILE_SIZES: usize = 1_000_000;
static OUTPUT_DIR: &str = "hashes";

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    //downloader::download_virusshare::download_all()?;
    downloader::download_virusshare::write_files()?;
    Ok(())
}
