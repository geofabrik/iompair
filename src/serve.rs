extern crate hyper;
extern crate clap;
extern crate rustc_serialize;
extern crate slippy_map_tiles;

use std::io::Read;
use std::fs;
use std::fs::File;
use std::path::Path;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;

use clap::ArgMatches;

//use rustc_serialize::json;

use slippy_map_tiles::Tile;

use utils::{URL, parse_url};

#[derive(Debug)]
enum IompairTileJsonError {
    OpenFileError(::std::io::Error),
    ReadFileError(::std::io::Error),
    InvalidJsonError(rustc_serialize::json::BuilderError),
    NoJSONObjectError,
    JsonEncoderError(rustc_serialize::json::EncoderError),
}

pub fn serve(options: &ArgMatches) {

    let port = options.value_of("port").unwrap().to_string();
    // TODO make path absolute
    let path = options.value_of("tc_path").or(options.value_of("ts_path")).unwrap().to_string();
    let use_tc: bool = options.value_of("tc_path").is_some();
    let maxzoom: u8 = options.value_of("maxzoom").unwrap().parse().unwrap();
    let urlprefix = options.value_of("urlprefix").unwrap_or(&format!("http://localhost:{}/", port)).to_string();
    
    // TODO read in tilejson file, change it and save it. Don't regenerate every request.

    println!("Serving on port {}", port);
    let uri = format!("127.0.0.1:{}", port);
    match Server::http(&uri[..]) {
        Err(e) => { println!("Error setting up server: {:?}", e); }
        Ok(server) => {
            let startup = server.handle(move |req: Request, res: Response| { base_handler(req, res, use_tc, &path, maxzoom, &urlprefix) });
            if let Err(e) = startup {
                println!("Error when starting server: {:?}", e);
            }
        }
    }
}

fn tilejson_contents(path: &str, prefix: Option<String>, urlprefix: &str, maxzoom: u8) -> Result<String, IompairTileJsonError> {
    // FIXME Remove the unwraps and replace with proper error handling
    let new_tiles = json::Json::from_str(&format!("[\"{}{{z}}/{{x}}/{{y}}.pbf\"]", urlprefix)).unwrap();
    let zoom_element = json::Json::U64(maxzoom as u64);

    // FIXME don't fall over if there is no file
    // TODO do proper std::path stuff here, instead of string concat
    let tilejson_path = match prefix {
        None => format!("{}/index.json", path),
        Some(prefix) => format!("{}/{}/index.json", path, prefix),
    };
    let mut f = try!(File::open(tilejson_path).map_err(IompairTileJsonError::OpenFileError));
    let mut s = String::new();
    try!(f.read_to_string(&mut s).map_err(IompairTileJsonError::ReadFileError));

    // Some back and forth to decode, replace and encode to get the new tilejson string
    let tilejson_0 = try!(json::Json::from_str(&s).map_err(IompairTileJsonError::InvalidJsonError));
    let mut tilejson = try!(tilejson_0.as_object().ok_or(IompairTileJsonError::NoJSONObjectError)).to_owned();
    tilejson.insert("tiles".to_owned(), new_tiles);
    tilejson.insert("maxzoom".to_owned(), zoom_element);
    let new_tilejson_contents: String = try!(json::encode(&tilejson).map_err(IompairTileJsonError::JsonEncoderError));

    Ok(new_tilejson_contents)
}

fn base_handler(req: Request, mut res: Response, use_tc: bool, path: &str, maxzoom: u8, urlprefix: &str) {
    let mut url: String = String::new();
    if let hyper::uri::RequestUri::AbsolutePath(ref u) = req.uri {
        url = u.clone();
    }
        
    match parse_url(&url, maxzoom) {
        URL::Tilejson(prefix) => {
            tilejson_handler(res, path, urlprefix, maxzoom, prefix);
        },
        URL::Invalid => {
            *res.status_mut() = hyper::status::StatusCode::NotFound;
        },
        URL::Tile(prefix, z, x, y, ext) => {
            tile_handler(res, use_tc, path, prefix, z, x, y, ext);
        }
    }
}

fn tile_handler(mut res: Response, use_tc: bool, path: &str, prefix: Option<String>, z: u8, x: u32, y: u32, ext: String) {
    let tile = Tile::new(z, x, y);
    let tile = try_or_err!(tile.ok_or("ERR"), res, format!("Error when turning z {} x {} y {} into tileobject", z, x, y));

    let this_prefix = match prefix {
        None => path.to_string(),
        Some(prefix) => format!("{}/{}", path, prefix),
    };

    let path = if use_tc { format!("{}/{}", this_prefix, tile.tc_path(ext)) } else { format!("{}/{}", this_prefix, tile.ts_path(ext)) };
    let this_tile_path = Path::new(&path);

    // This is a stupid bit of hackery to ensure that s is initialised to /something/
    let mut vector_tile_contents: Vec<u8> = Vec::new();
    
    if this_tile_path.exists() {
        let mut file = try_or_err!(fs::File::open(this_tile_path), res, format!("Error when opening file {:?}", this_tile_path));
        try_or_err!(file.read_to_end(&mut vector_tile_contents), res, format!("Error when trying to send vectortile contents to client"));
    } else {
        // File not found. This can happen in the middle of the ocean or something
        // If we return a 404 then Kosmtik throws an error, instead return a 200. The results will
        // be empty anyway
    }

    res.headers_mut().set(ContentType(Mime(TopLevel::Application, SubLevel::Ext("x-protobuf".to_owned()), vec![])));
    res.send(&vector_tile_contents).unwrap_or_else(|e| {
        println!("Error when trying to send tilejson to client: {:?}", e);
    });

}

fn tilejson_handler(mut res: Response, path: &str, urlprefix: &str, maxzoom: u8, prefix: Option<String>) {
    match tilejson_contents(path, prefix, urlprefix, maxzoom) {
        Err(e) => {
            println!("Error when reading tilejson file to serve up: {:?}", e);
            *res.status_mut() = hyper::status::StatusCode::InternalServerError;
        },
        Ok(json) => {
            res.send(json.as_bytes()).unwrap_or_else(|e| {
                println!("Error when trying to send tilejson to client: {:?}", e);
            });
        }
    };
}
