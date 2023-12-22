

use log::trace;
use rusqlite::params;

use crate::{TABLE_NAME, DATABASE};


pub fn create_table(connection: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    connection.execute(&format!("CREATE TABLE IF NOT EXISTS {} (
        id INTEGER PRIMARY KEY,
        hash TEXT NOT NULL UNIQUE
    )", TABLE_NAME), [])?;
    Ok(())
}

pub fn insert_hashes(connection: &mut rusqlite::Connection, hashes: &Vec<String>) -> Result<(), rusqlite::Error> {
    let transaction = connection.transaction()?;
    for hash in hashes {
        trace!("Inserting {hash}");
        transaction.execute(&format!("INSERT OR IGNORE INTO {} (hash) VALUES (?1)", TABLE_NAME), params![hash])?;
    }
    transaction.commit()?;
    Ok(())
}

pub fn get_hashes(connection: &rusqlite::Connection, bottom_index: usize, top_index: usize) -> Result<Vec<String>, rusqlite::Error> {
    let mut sql = connection.prepare(&format!("SELECT hash FROM {} WHERE id >= ?1 AND id < ?2", TABLE_NAME))?;
    let hashes: Result<Vec<String>, rusqlite::Error> = sql.query_map(params![bottom_index, top_index], |row| row.get(0))?.collect();
    let out = hashes.map_err(|_| {}).expect("Failed to map errors");
    Ok(out)
}

pub fn create_pool() -> Result<rusqlite::Connection, rusqlite::Error> {
    rusqlite::Connection::open(DATABASE)
}