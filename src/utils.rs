extern crate hyper;

use std::io::Read;
use std::path::Path;
use std::fs;
use std::io::Write;

use hyper::Client;

/// Given a URL, it'll download the URL and return the bytes. None if there was an error.
pub fn download_url(url: &str) -> Option<Vec<u8>> {
    let client = Client::new();
    let mut result = client.get(url).send();
    if result.is_err() { return None; }
    let mut result = result.unwrap();
    if result.status != hyper::status::StatusCode::Ok {
        return None;
    }

    let mut file_contents: Vec<u8> = Vec::new();
    result.read_to_end(&mut file_contents);

    Some(file_contents)
}

/// Saves this bytes to this path
/// All errors are silently ignored
pub fn save_to_file(path: &Path, bytes: &Vec<u8>) {
    let parent_directory = path.parent();
    if parent_directory.is_none() { return; }
    let parent_directory = parent_directory.unwrap();
    if ! parent_directory.exists() {
        fs::create_dir_all(parent_directory);
    }

    let mut file = fs::File::create(path);
    if file.is_err() { return; }
    let mut file = file.unwrap();
    file.write_all(bytes);

}
