extern crate hyper;
extern crate regex;

use regex::Regex;

use std::io::Read;
use std::path::Path;
use std::fs;
use std::io::Write;
use std::io;
use std::time::Duration;

use hyper::Client;

// Do something that returns a Result. If there's an error, the response will be set to an
// appropriate code, optionally something printed to stdout, and the handler will return.
macro_rules! try_or_err {

    ($e:expr, $res:ident) => (match $e {
        Ok(e) => e,
        Err(e) => {
            println!("Error: {:?}", e);
            *$res.status_mut() = hyper::status::StatusCode::InternalServerError;
            return;
        }
    });

    ($e:expr, $res:ident, $errmsg:expr) => (match $e {
        Ok(e) => e,
        Err(e) => {
            println!("{} {:?}", $errmsg, e);
            *$res.status_mut() = hyper::status::StatusCode::InternalServerError;
            return;
        }
    });

    ($e:expr, $res:ident, $errmsg:expr, Ok => $ok:block) => (match $e {
        Ok(_) => $ok,
        Err(e) => {
            println!("{} {:?}", $errmsg, e);
            *$res.status_mut() = hyper::status::StatusCode::InternalServerError;
            return;
        }
    });
}

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


/// Given a URL, it'll download the URL and return the bytes, or an error of what happened. If
/// there's an error, it tries at most `num_tries` times.
pub fn download_url(url: &str, num_tries: u8) -> Result<Vec<u8>, IompairError> {
    // Do first download, which ensures result is always initialised
    let mut result = download_url_single(url);

    for _ in 1..num_tries {
        result = download_url_single(url);
        if result.is_ok() {
            // Successful download! Bail out early.
            return result;
        }
        // otherwise just try again at the loop
    }

    // If we've gotten to hear it has always failed and we've tried enough. So just return that
    // error
    result
}

fn download_url_single(url: &str) -> Result<Vec<u8>, IompairError> {
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
    let contents = try!(download_url(url, 10));

    save_to_file(path, &contents)
}

#[derive(Debug, PartialEq, Eq)]
pub enum URL {
    Invalid,
    Tilejson(Option<String>),
    Tile(Option<String>, u8, u32, u32, String),
}


pub fn parse_url(url: &str, maxzoom: u8) -> URL {

    // Macro which returns URL::Invalid if the Option<T> is None. Makes it easier for early return
    macro_rules! or_invalid {
        ($e:expr) => (match $e { Some(e) => e, None => return URL::Invalid });
    }
    // FIXME reuse regex

    if let Some(caps) = Regex::new("^(/(?P<prefix>[a-zA-Z0-9_-]+))?/index.json$").unwrap().captures(url) {
        URL::Tilejson(caps.name("prefix").map(|x| x.to_string()))
    } else {
        let re = Regex::new("^(/(?P<prefix>[a-zA-Z0-9_-]+))?/(?P<z>[0-9]?[0-9])/(?P<x>[0-9]+)/(?P<y>[0-9]+)\\.(?P<ext>.{3,4})$").unwrap();
        if let Some(caps) = re.captures(url) {
            let z: u8 = or_invalid!(or_invalid!(caps.name("z")).parse().ok());
            if z > maxzoom {
                URL::Invalid
            } else {
                let x: u32 = or_invalid!(or_invalid!(caps.name("x")).parse().ok());
                let y: u32 = or_invalid!(or_invalid!(caps.name("y")).parse().ok());
                let ext: String = or_invalid!(caps.name("ext")).to_owned();
                URL::Tile(caps.name("prefix").map(|x| x.to_string()), z, x, y, ext)
            }
        } else {
            URL::Invalid
        }
    }
}


mod test {
    #[test]
    fn test_url_parse() {
        use super::{parse_url, URL};

        assert_eq!(parse_url("/", 22), URL::Invalid);
        assert_eq!(parse_url("/robots.txt", 22), URL::Invalid);
        assert_eq!(parse_url("/index.json", 22), URL::Tilejson(None));
        assert_eq!(parse_url("/2/12/12.png", 22), URL::Tile(None, 2, 12, 12, "png".to_owned()));
        assert_eq!(parse_url("/2/12/12.png", 1), URL::Invalid);

        assert_eq!(parse_url("/foobar/index.json", 22), URL::Tilejson(Some("foobar".to_string())));
        assert_eq!(parse_url("/foobar/2/12/12.png", 22), URL::Tile(Some("foobar".to_string()), 2, 12, 12, "png".to_owned()));
        assert_eq!(parse_url("/HELLO_there-number-3/2/12/12.png", 22), URL::Tile(Some("HELLO_there-number-3".to_string()), 2, 12, 12, "png".to_owned()));
        assert_eq!(parse_url("/no spaces/2/12/12.png", 22), URL::Invalid);
        assert_eq!(parse_url("bad bad bad no spaces/2/12/12.png", 22), URL::Invalid);

    }


}

