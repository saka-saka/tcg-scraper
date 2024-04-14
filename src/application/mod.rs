mod application;
mod bigweb;
mod one_piece;
mod pokemon;
mod ws;
mod yugioh;

use std::{io::Write, path::Path};

pub use application::*;

async fn download<T: AsRef<Path>>(url: url::Url, save_path: T) -> Result<(), crate::error::Error> {
    let result = reqwest::get(url).await?;
    let paths = result.url().path_segments().unwrap();
    let file_name = paths.last().unwrap();
    let save_path = save_path.as_ref().join(file_name);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(save_path)?;
    let bytes = result.bytes().await?;
    let _ = file.write(&bytes)?;
    Ok(())
}
