extern crate hyper;
extern crate regex;

use regex::Regex;

use std::io::Read;
use std::path::Path;
use std::fs;
use std::io::Write;
use std::io;
use std::time::Duration;
use std::fmt;

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

/// A prefix for a URL path
/// Like /foo__bar/index.json which is the concat of both foo and bar levels.
/// /index.json would be no other layers invovled
#[derive(Debug, PartialEq, Eq)]
pub struct URLPathPrefix {
    parts: Option<Vec<String>>
}

impl URLPathPrefix {
    /// Construct a new URLPathPrefix
    fn new<S: Into<String>>(parts: Option<Vec<S>>) -> Self {
        // Tried to have 
        // let new_parts: Option<Vec<String>> = parts.map(|x| x.iter().map(|y| y.into()).collect());
        // but that didn't work
        let new_parts: Option<Vec<String>> = match parts {
            None => None,
            Some(p) => {
                let mut new: Vec<String> = Vec::with_capacity(p.len());
                for x in p {
                    let x: String = x.into();
                    new.push(x);
                }
                Some(new)
            }
        };
        URLPathPrefix{ parts: new_parts }
    }

    /// Shortcut to create a URLPathPrefix with no prefix
    fn none() -> Self { URLPathPrefix{ parts: None } }

    /// Shortcut to create a URLPathPrefix with the following parts
    fn parts<S: Into<String>>(parts: Vec<S>) -> Self {
        URLPathPrefix::new(Some(parts))
    }

    /// Construct a URLPathPrefix from a path string, like "foo__bar" or ""
    fn parse<S>(s: Option<S>) -> Self where S: Into<String> {
        match s {
            None => URLPathPrefix{ parts: None },
            Some(mystring) => {
                URLPathPrefix{ parts: Some(mystring.into().split("__").map(|x| x.to_string()).filter(|x| x != "").collect::<Vec<String>>()) }
            }
        }
    }

    pub fn path_with_trailing_slash(&self) -> String {
        match self.parts {
            None => "".to_string(),
            Some(ref p) => format!("{}/", p.join("__")),
        }
    }

    /// Given a directory, return all the other directories that this URLPathPrefix referrs to
    pub fn paths(&self, path: &str) -> Vec<String> {
        match self.parts {
            None => vec![path.to_string()],
            Some(ref p) => p.iter().map(|x| format!("{}/{}", path, x)).collect(),
        }
    }

}

impl fmt::Display for URLPathPrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parts {
            None => write!(f, ""),
            Some(ref parts) => write!(f, "{}", parts.join("__")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum URL {
    Invalid,
    Tilejson(URLPathPrefix),
    Tile(URLPathPrefix, u8, u32, u32, String),
}


pub fn parse_url(url: &str, maxzoom: u8) -> URL {

    // Macro which returns URL::Invalid if the Option<T> is None. Makes it easier for early return
    macro_rules! or_invalid {
        ($e:expr) => (match $e { Some(e) => e, None => return URL::Invalid });
    }
    // FIXME reuse regex

    if let Some(caps) = Regex::new("^(/(?P<prefix>[a-zA-Z0-9_-]+))?/index.json$").unwrap().captures(url) {
        URL::Tilejson(URLPathPrefix::parse(caps.name("prefix")))
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
                URL::Tile(URLPathPrefix::parse(caps.name("prefix")), z, x, y, ext)
            }
        } else {
            URL::Invalid
        }
    }
}


mod test {
    #[test]
    fn test_urlprefix() {
        use super::URLPathPrefix;

        // Some variables to prevent "cannot infer type"
        let none_string: Option<String> = None;
        let none_vec: Option<Vec<String>> = None;
        let empty_vec: Vec<String> = vec![];

        assert_eq!(URLPathPrefix::parse(none_string.clone()), URLPathPrefix::new(none_vec));
        assert_eq!(URLPathPrefix::parse(Some("")), URLPathPrefix::parts(empty_vec));
        assert_eq!(URLPathPrefix::parse(Some("abc")), URLPathPrefix::parts(vec!["abc".to_string()]));
        assert_eq!(URLPathPrefix::parse(Some("abc_xyz")), URLPathPrefix::parts(vec!["abc_xyz".to_string()]));
        assert_eq!(URLPathPrefix::parse(Some("abc__xyz")), URLPathPrefix::parts(vec!["abc".to_string(), "xyz".to_string()]));

        assert_eq!(URLPathPrefix::parse(none_string.clone()).paths("/tmp"), vec!["/tmp"]);
        assert_eq!(URLPathPrefix::parse(Some("abc")).paths("/tmp"), vec!["/tmp/abc"]);
        assert_eq!(URLPathPrefix::parse(Some("abc__xyz")).paths("/tmp"), vec!["/tmp/abc", "/tmp/xyz"]);
        assert_eq!(URLPathPrefix::parse(Some("abc__xyz__foo__bar")).paths("/tmp"), vec!["/tmp/abc", "/tmp/xyz", "/tmp/foo", "/tmp/bar"]);
    }

    #[test]
    fn test_url_parse() {
        use super::{parse_url, URL, URLPathPrefix};


        assert_eq!(parse_url("/", 22), URL::Invalid);
        assert_eq!(parse_url("/robots.txt", 22), URL::Invalid);
        assert_eq!(parse_url("/index.json", 22), URL::Tilejson(URLPathPrefix::none()));
        assert_eq!(parse_url("/2/12/12.png", 22), URL::Tile(URLPathPrefix::none(), 2, 12, 12, "png".to_owned()));
        assert_eq!(parse_url("/2/12/12.png", 1), URL::Invalid);

        assert_eq!(parse_url("/foobar/index.json", 22), URL::Tilejson(URLPathPrefix::parts(vec!["foobar"])));
        assert_eq!(parse_url("/foobar/2/12/12.png", 22), URL::Tile(URLPathPrefix::parts(vec!["foobar"]), 2, 12, 12, "png".to_owned()));
        assert_eq!(parse_url("/HELLO_there-number-3/2/12/12.png", 22), URL::Tile(URLPathPrefix::parts(vec!["HELLO_there-number-3"]), 2, 12, 12, "png".to_owned()));
        assert_eq!(parse_url("/no spaces/2/12/12.png", 22), URL::Invalid);
        assert_eq!(parse_url("bad bad bad no spaces/2/12/12.png", 22), URL::Invalid);

        assert_eq!(parse_url("/foo__bar/index.json", 22), URL::Tilejson(URLPathPrefix::parts(vec!["foo", "bar"])));
        assert_eq!(parse_url("/foo__bar/0/0/0.png", 22), URL::Tile(URLPathPrefix::parts(vec!["foo", "bar"]), 0, 0, 0, "png".to_string()));
        assert_eq!(parse_url("/bar__foo/0/0/0.png", 22), URL::Tile(URLPathPrefix::parts(vec!["bar", "foo"]), 0, 0, 0, "png".to_string()));
        assert_eq!(parse_url("/foo__bar__baz/0/0/0.png", 22), URL::Tile(URLPathPrefix::parts(vec!["foo", "bar", "baz"]), 0, 0, 0, "png".to_string()));

    }


}

