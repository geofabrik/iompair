extern crate hyper;

use std::io::Read;
use std::path::Path;
use std::fs;
use std::io::Write;
use std::io;
use std::time::Duration;

use hyper::Client;

#[derive(Debug)]
pub enum IompairError {
    DownloadError(hyper::Error),
    Non200ResponseError(hyper::status::StatusCode),
    ReadResponseError(io::Error),
    
    NoParentDirectoryError,
    OpenFileError(io::Error),
    WriteToFileError(io::Error),
    CreateDirsError(io::Error),
}

// TODO should we impl error::Error for IompairError ? Why?

//impl From<hyper::Error> for IompairError {
//    fn from(err: hyper::Error) -> IompairError { IompairError::DownloadError(err)  }
//}


/// Given a URL, it'll download the URL and return the bytes, or an error of what happened
pub fn download_url(url: &str) -> Result<Vec<u8>, IompairError> {
    let mut client = Client::new();
    
    // set the timeout to be 1 day
    client.set_read_timeout(Some(Duration::new(1 * 24 * 60 * 60, 0)));
    
    let mut result = try!(client.get(url).send().map_err(IompairError::DownloadError));
    if result.status != hyper::status::StatusCode::Ok {
        return Err(IompairError::Non200ResponseError(result.status));
    }

    let mut file_contents: Vec<u8> = Vec::new();
    try!(result.read_to_end(&mut file_contents).map_err(IompairError::ReadResponseError));

    Ok(file_contents)
}

/// Saves this bytes to this path
/// Errors are returned
pub fn save_to_file(path: &Path, bytes: &Vec<u8>) -> Result<(), IompairError> {
    let parent_directory = try!(path.parent().ok_or(IompairError::NoParentDirectoryError));
    if ! parent_directory.exists() {
        try!(fs::create_dir_all(parent_directory).map_err(IompairError::CreateDirsError));
    }

    let mut file = try!(fs::File::create(path).map_err(IompairError::OpenFileError));
    try!(file.write_all(bytes).map_err(IompairError::WriteToFileError));

    Ok(())
}

/// Downloads the URL and if it went OK, saves the contents to path. Returns Error if something
/// went wrong.
pub fn download_url_and_save_to_file(url: &str, path: &Path) -> Result<(), IompairError> {
    let contents = try!(download_url(url));

    save_to_file(path, &contents)
}
