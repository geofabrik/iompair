extern crate iron;
extern crate router;
extern crate hyper;
extern crate clap;
extern crate rustc_serialize;
extern crate slippy_map_tiles;

use std::io::Read;
use std::fs;
use std::path::Path;
use std::process::exit;

use iron::{Iron, Request, Response, IronResult};
use iron::status;
use router::{Router};
use hyper::Client;
use clap::ArgMatches;
use rustc_serialize::json;

use slippy_map_tiles::Tile;

use utils::download_url_and_save_to_file;

pub fn cache(options: &ArgMatches) {

    let port = options.value_of("port").unwrap();
    let upstream_url = options.value_of("upstream_url").unwrap().to_string();
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    let maxzoom = options.value_of("maxzoom").and_then(|x| { x.parse::<u8>().ok() });
    // TODO make tc_path absolute

    let mut router = Router::new();

    // get the tilejson details
    let client = Client::new();

    // FIXME split this into functions
    // FIXME use the URLs in the tile json for fetching tiles

    // Grab the upstream tilejson, but replace the `tiles` key with the url of this

    // This is the parsed, new value for tiles
    let new_tiles = json::Json::from_str(&format!("[\"http://localhost:{}/{{z}}/{{x}}/{{y}}.pbf\"]", port)).unwrap();

    // TODO return appropriate error from upstream
    let mut result = client.get(&format!("{}/index.json", upstream_url)).send().unwrap();
    
    // Some back and forth to decode, replace and encode to get the new tilejson string
    let mut tilejson_contents = String::new();
    result.read_to_string(&mut tilejson_contents).unwrap();
    let tilejson_0 = json::Json::from_str(&tilejson_contents);
    if tilejson_0.is_err() {
        println!("TileJSON at {}/index.json is not a valid JSON file, error: {:?}. Contents: {:?} Exiting.", upstream_url, tilejson_0, tilejson_contents);
        exit(1);
    }
    let tilejson_0 = tilejson_0.unwrap();

    let mut tilejson = tilejson_0.as_object().unwrap().to_owned();
    tilejson.insert("tiles".to_owned(), new_tiles);

    if let Some(z) = maxzoom {
        tilejson.insert("maxzoom".to_owned(), json::Json::U64(z as u64));
    }

    let new_tilejson_contents: String = json::encode(&tilejson).unwrap();

    router.get("/index.json", move |r: &mut Request| tilejson_handler(r, &new_tilejson_contents));

    fn tilejson_handler(_: &mut Request, tilejson_contents: &str) -> IronResult<Response> {
        Ok(Response::with((status::Ok, tilejson_contents)))
    }

    // Handler for tiles
    router.get("/:z/:x/:y", move |r: &mut Request| tile_handler(r, &upstream_url, &tc_path));

    fn tile_handler(req: &mut Request, upstream_url: &str, tc_path: &str) -> IronResult<Response> {
        // FIXME parse properly and return 403 if wrong
        let z: u8 = req.extensions.get::<Router>().unwrap().find("z").unwrap().parse().unwrap();
        let x: u32 = req.extensions.get::<Router>().unwrap().find("x").unwrap().parse().unwrap();

        let y_full = req.extensions.get::<Router>().unwrap().find("y").unwrap();
        let y_parts = y_full.split(".").nth(0).unwrap();
        let y: u32 = y_parts.parse().unwrap();

        let tile = Tile::new(z, x, y).unwrap();
        let path = format!("{}/{}", tc_path, tile.tc_path("pbf"));
        let this_tile_tc_path = Path::new(&path);

        // This is a stupid bit of hackery to ensure that s is initialised to /something/
        let mut vector_tile_contents: Vec<u8> = Vec::new();
        
        if this_tile_tc_path.exists() {
            let mut file = fs::File::open(this_tile_tc_path).unwrap();
            file.read_to_end(&mut vector_tile_contents);
        } else {
            download_url_and_save_to_file(&format!("{}/{}/{}/{}.pbf", upstream_url, z, x, y), this_tile_tc_path);
        }

        // FIXME correct Content-Type
        Ok(Response::with((status::Ok, vector_tile_contents)))

    }

    println!("Serving on port {}", port);
    Iron::new(router).http(&*format!("localhost:{}", port)).unwrap();
}
