extern crate hyper;
extern crate clap;
extern crate rustc_serialize;
extern crate slippy_map_tiles;

use std::io::Read;
use std::fs;
use std::path::Path;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::header::{ContentType, CacheDirective, CacheControl};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::Client;
use hyper::status::StatusCode;

use clap::ArgMatches;
use rustc_serialize::json;

use slippy_map_tiles::Tile;

use utils::{download_url, save_to_file, parse_url, URL};

macro_rules! try_or_ret {
    ($e:expr, $msg:expr) => ( match $e {
        Ok(e) => e,
        Err(e) => {
            println!("{} {:?}", $msg, e);
            return;
        },
    });
}

pub fn cache(options: &ArgMatches) {

    let port = options.value_of("port").unwrap();
    let upstream_url = options.value_of("upstream_url").unwrap().to_string();
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    let maxzoom = options.value_of("maxzoom").and_then(|x| { x.parse::<u8>().ok() });
    // TODO make tc_path absolute

    // get the tilejson details
    let client = Client::new();

    //let mut router = Router::new();

    // FIXME split this into functions
    // FIXME use the URLs in the tile json for fetching tiles

    // Grab the upstream tilejson, but replace the `tiles` key with the url of this

    // This is the parsed, new value for tiles
    let new_tiles = json::Json::from_str(&format!("[\"http://localhost:{}/{{z}}/{{x}}/{{y}}.pbf\"]", port)).unwrap();

    // TODO return appropriate error from upstream
    let mut result = try_or_ret!(client.get(&format!("{}/index.json", upstream_url)).send(), "Error getting TileJSON from upstream");
    
    // Some back and forth to decode, replace and encode to get the new tilejson string
    let mut tilejson_contents = String::new();
    try_or_ret!(result.read_to_string(&mut tilejson_contents), "Error when trying to read tilejson contents");
    let tilejson_0 = try_or_ret!(json::Json::from_str(&tilejson_contents), "TileJSON at {}/index.json is not a valid JSON file");

    let mut tilejson = try_or_ret!(tilejson_0.as_object().ok_or("ERR"), "Error when trying to read tilejson contents").to_owned();
    tilejson.insert("tiles".to_owned(), new_tiles);

    if let Some(z) = maxzoom {
        tilejson.insert("maxzoom".to_owned(), json::Json::U64(z as u64));
    }

    let new_tilejson_contents: String = try_or_ret!(json::encode(&tilejson), "Error when trying to create tilejson contents");

    fn tilejson_handler(res: Response, tilejson_contents: &str) {
        res.send(tilejson_contents.as_bytes()).unwrap_or_else(|e| {
            println!("Error when trying to send tilejson to client: {:?}", e);
        });
    }

    // Handler for tiles

    fn tile_handler(mut res: Response, tc_path: &str, z: u8, x: u32, y: u32, upstream_url: &str) {
        let tile = match Tile::new(z, x, y) {
            None => {
                *res.status_mut() = hyper::status::StatusCode::BadRequest;
                res.send(format!("Invalid tile number {}/{}/{}", z, x, y).as_bytes()).ok();
                return;
            },
            Some(t) => t,
        };
        let path = format!("{}/{}", tc_path, tile.tc_path("pbf"));
        let this_tile_tc_path = Path::new(&path);

        // This is a stupid bit of hackery to ensure that s is initialised to /something/
        let mut vector_tile_contents: Vec<u8> = Vec::new();
        
        if this_tile_tc_path.exists() {
            let mut file = try_or_err!(fs::File::open(this_tile_tc_path), res, format!("Couldn't open tile {:?}", this_tile_tc_path));
            try_or_err!(file.read_to_end(&mut vector_tile_contents), res, "Error when trying to send vectortile contents to client");
            println!("Cache hit {}/{}/{}", z, x, y);
        } else {
            try_or_err!(download_url(&format!("{}/{}/{}/{}.pbf", upstream_url, z, x, y), 10), res,
                format!("Cache miss {}/{}/{} and error downloading file", z, x, y),
                Ok => {
                    try_or_err!(save_to_file(this_tile_tc_path, &vector_tile_contents), res,
                        "Error when trying to save file to cache",
                        Ok => { println!("Cache miss {}/{}/{} Downloaded and saved in {:?}", z, x, y, this_tile_tc_path); }
                    );
                }
            );
        }

        *res.status_mut() = StatusCode::Ok;
        res.headers_mut().set(CacheControl(vec![CacheDirective::Private, CacheDirective::NoCache, CacheDirective::MaxAge(0)]));
        res.headers_mut().set(ContentType(Mime(TopLevel::Application, SubLevel::Ext("x-protobuf".to_owned()), vec![])));
        res.send(&vector_tile_contents).ok();
    }

    fn base_handler(req: Request, mut res: Response, tc_path: &str, upstream_url: &str, tilejson_contents: &str) {
        let mut url: String = String::new();
        if let hyper::uri::RequestUri::AbsolutePath(ref u) = req.uri {
            url = u.clone();
        }
            
        match parse_url(&url, 22) {
            URL::Tilejson(prefix) => {
                tilejson_handler(res, tilejson_contents);
            },
            URL::Invalid => {
                *res.status_mut() = hyper::status::StatusCode::NotFound;
            },
            URL::Tile(prefix, z, x, y, _) => {
                tile_handler(res, tc_path, z, x, y, upstream_url);
            }
        }
    }

    println!("Serving on port {}", port);
    match Server::http(&*format!("localhost:{}", port)) {
        Err(e) => { println!("Couldn't open port: {:?}", e); return },
        Ok(s) => {
            s.handle(move |req: Request, res: Response| { base_handler(req, res, &tc_path, &upstream_url, &new_tilejson_contents) }).ok();
        },
    };
}
