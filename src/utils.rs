extern crate hyper;

use std::io::Read;

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
