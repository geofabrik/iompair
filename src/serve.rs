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
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    let maxzoom: u8 = options.value_of("maxzoom").unwrap().parse().unwrap();
    let urlprefix = options.value_of("urlprefix").unwrap_or(&format!("http://localhost:{}/", port)).to_string();
    // TODO make tc_path absolute
    
    // TODO read in tilejson file, change it and save it. Don't regenerate every request.

    println!("Serving on port {}", port);
    let uri = format!("127.0.0.1:{}", port);
    match Server::http(&uri[..]) {
        Err(e) => { println!("Error setting up server: {:?}", e); }
        Ok(server) => {
            let startup = server.handle(move |req: Request, res: Response| { base_handler(req, res, &tc_path, maxzoom, &urlprefix) });
            if let Err(e) = startup {
                println!("Error when starting server: {:?}", e);
            }
        }
    }
}

fn tilejson_contents(tc_path: &str, urlprefix: &str, maxzoom: u8) -> Result<String, IompairTileJsonError> {
    // FIXME Remove the unwraps and replace with proper error handling
    let new_tiles = json::Json::from_str(&format!("[\"{}{{z}}/{{x}}/{{y}}.pbf\"]", urlprefix)).unwrap();
    let zoom_element = json::Json::U64(maxzoom as u64);

    // FIXME don't fall over if there is no file
    let mut f = try!(File::open(format!("{}/index.json", tc_path)).map_err(IompairTileJsonError::OpenFileError));
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

fn base_handler(req: Request, mut res: Response, tc_path: &str, maxzoom: u8, urlprefix: &str) {
    let mut url: String = String::new();
    if let hyper::uri::RequestUri::AbsolutePath(ref u) = req.uri {
        url = u.clone();
    }
        
    match parse_url(&url, maxzoom) {
        URL::Tilejson => {
            tilejson_handler(res, tc_path, urlprefix, maxzoom);
        },
        URL::Invalid => {
            *res.status_mut() = hyper::status::StatusCode::NotFound;
        },
        URL::Tile(z, x, y, ext) => {
            tile_handler(res, tc_path, z, x, y, ext);
        }
    }
}

fn tile_handler(mut res: Response, tc_path: &str, z: u8, x: u32, y: u32, ext: String) {
    let tile = Tile::new(z, x, y);
    let tile = match tile {
        None => {
            *res.status_mut() = hyper::status::StatusCode::NotFound;
            println!("Error when turning z {} x {} y {} into tileobject", z, x, y);
            return;
        },
        Some(t) => { t },
    };

    let path = format!("{}/{}", tc_path, tile.tc_path(ext));
    let this_tile_tc_path = Path::new(&path);

    // This is a stupid bit of hackery to ensure that s is initialised to /something/
    let mut vector_tile_contents: Vec<u8> = Vec::new();
    
    if this_tile_tc_path.exists() {
        let mut file = match fs::File::open(this_tile_tc_path) {
            Err(e) => {
                *res.status_mut() = hyper::status::StatusCode::InternalServerError;
                println!("Error when opening file {:?}: {:?}", this_tile_tc_path, e);
                return;
            },
            Ok(f) => { f },
        };
        match file.read_to_end(&mut vector_tile_contents) {
            Ok(_) => {},
            Err(e) => {
                println!("Error when trying to send vectortile contents to client: {:?}", e);
                *res.status_mut() = hyper::status::StatusCode::InternalServerError;
                return;
            },
        }
    } else {
        *res.status_mut() = hyper::status::StatusCode::NotFound;
    }

    res.headers_mut().set(ContentType(Mime(TopLevel::Application, SubLevel::Ext("x-protobuf".to_owned()), vec![])));
    res.send(&vector_tile_contents).unwrap_or_else(|e| {
        println!("Error when trying to send tilejson to client: {:?}", e);
    });

}

fn tilejson_handler(mut res: Response, tc_path: &str, urlprefix: &str, maxzoom: u8) {
    match tilejson_contents(tc_path, urlprefix, maxzoom) {
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
