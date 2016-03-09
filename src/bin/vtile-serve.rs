extern crate hyper;
extern crate clap;
extern crate rustc_serialize;
extern crate slippy_map_tiles;
extern crate regex;

use std::io::Read;
use std::fs;
use std::path::Path;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};

use regex::Regex;

use clap::{Arg, App};

//use rustc_serialize::json;

use slippy_map_tiles::Tile;

fn main() {

    let options = App::new("vtiles-serve")
        .arg(Arg::with_name("port").short("p").long("port")
             .takes_value(true).required(true)
             .help("Port to listen on").value_name("PORT"))
        .arg(Arg::with_name("tc_path").short("c").long("tc-path")
             .takes_value(true).required(true)
             .help("Directory to use as a tile cache.").value_name("PATH"))
        .get_matches();

    let port = options.value_of("port").unwrap();
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    // TODO make tc_path absolute

    println!("Serving on port {}", port);
    let uri = format!("127.0.0.1:{}", port);
    Server::http(&uri[..]).unwrap().handle(move |req: Request, res: Response| { base_handler(req, res, &tc_path) }).unwrap();
}

fn base_handler(req: Request, mut res: Response, tc_path: &str) {
    let mut url: String = String::new();
    if let hyper::uri::RequestUri::AbsolutePath(ref u) = req.uri {
        url = u.clone();
    }
        
    match parse_url(&url) {
        URL::Tilejson => {
            tilejson_handler(res, tc_path);
        },
        URL::Invalid => {
            *res.status_mut() = hyper::status::StatusCode::NotFound;
        },
        URL::Tile(z, x, y, ext) => {
            tile_handler(res, tc_path, z, x, y, ext);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum URL {
    Invalid,
    Tilejson,
    Tile(u8, u32, u32, String),
}


fn parse_url(url: &str) -> URL {
    // FIXME reuse regex
    if url == "/index.json" {
        URL::Tilejson
    } else {
        let re = Regex::new("/(?P<z>[0-9]?[0-9])/(?P<x>[0-9]+)/(?P<y>[0-9]+)\\.(?P<ext>.{3,4})").unwrap();
        if let Some(caps) = re.captures(url) {
            let z: u8 = caps.name("z").unwrap().parse().unwrap();
            let x: u32 = caps.name("x").unwrap().parse().unwrap();
            let y: u32 = caps.name("y").unwrap().parse().unwrap();
            let ext: String = caps.name("ext").unwrap().to_owned();
            URL::Tile(z, x, y, ext)
        } else {
            URL::Invalid
        }
    }
}


fn tile_handler(mut res: Response, tc_path: &str, z: u8, x: u32, y: u32, ext: String) {
    let tile = Tile::new(z, x, y);
    if tile.is_none() {
        *res.status_mut() = hyper::status::StatusCode::NotFound;
        return;
    }


    let tile = tile.unwrap();
    let path = format!("{}/{}", tc_path, tile.tc_path(ext));
    let this_tile_tc_path = Path::new(&path);

    // This is a stupid bit of hackery to ensure that s is initialised to /something/
    let mut vector_tile_contents: Vec<u8> = Vec::new();
    
    if this_tile_tc_path.exists() {
        let mut file = fs::File::open(this_tile_tc_path).unwrap();
        file.read_to_end(&mut vector_tile_contents);
    } else {
        *res.status_mut() = hyper::status::StatusCode::NotFound;
    }

    res.headers_mut().set(ContentType(Mime(TopLevel::Application, SubLevel::Ext("x-protobuf".to_owned()), vec![])));
    res.send(&vector_tile_contents);

}

fn tilejson_handler(mut res: Response, tc_path: &str) {
    let mut f = fs::File::open(format!("{}/index.json", tc_path)).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s);
    res.send(s.as_bytes());
}

mod test {
    #[test]
    fn test_url_parse() {
        use super::{parse_url, URL};

        assert_eq!(parse_url("/"), URL::Invalid);
        assert_eq!(parse_url("/index.json"), URL::Tilejson);
        assert_eq!(parse_url("/2/12/12.png"), URL::Tile(2, 12, 12, "png".to_owned()));

    }


}
