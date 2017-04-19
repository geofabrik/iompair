extern crate hyper;
extern crate clap;
extern crate rustc_serialize;
extern crate slippy_map_tiles;

use std::io::Read;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;

use clap::ArgMatches;

use slippy_map_tiles::Tile;

use utils::{save_to_file, download_url, URL, parse_url, URLPathPrefix, merge_vector_tiles, DirectoryLayout, IompairTileJsonError, download_url_and_save_to_file};

pub fn serve(options: &ArgMatches) {

    let port = options.value_of("port").unwrap().to_string();
    // TODO make path absolute
    let path = options.value_of("tc_path").or(options.value_of("ts_path")).or(options.value_of("zxy_path")).unwrap().to_string();
    let path_format = if options.is_present("tc_path") { DirectoryLayout::TCPath } else if options.is_present("ts_path") { DirectoryLayout::TSPath } else if options.is_present("zxy_path") { DirectoryLayout::ZXYPath } else { unreachable!() };
    let maxzoom: u8 = options.value_of("maxzoom").unwrap().parse().unwrap();
    let urlprefix = options.value_of("urlprefix").unwrap_or(&format!("http://localhost:{}/", port)).to_string();
    let verbose = options.is_present("verbose");
    
    let upstreams = parse_out_upstreams(options.values_of("upstream_url"));

    ensure_tilejson_files_exist(&path, &upstreams);

    println!("Serving on port {} with the following upstreams {:?}", port, upstreams);
    let uri = format!("127.0.0.1:{}", port);
    match Server::http(&uri[..]) {
        Err(e) => { println!("Error setting up server: {:?}", e); }
        Ok(server) => {
            let startup = server.handle(move |req: Request, res: Response| { base_handler(req, res, path_format, &path, maxzoom, &urlprefix, verbose, &upstreams) });
            if let Err(e) = startup {
                println!("Error when starting server: {:?}", e);
            }
        }
    }
}

/// Look at all the upstreams specified, and if the index.json (or metadata.json) doesn't exist,
/// download it from the upstream
fn ensure_tilejson_files_exist(path: &str, upstreams: &HashMap<String, String>) {
    for (prefix, upstream_url) in upstreams {
        let tilejson_url = format!("{}/index.json", upstream_url);
        let tilejson_path = if Path::new(&format!("{}/{}/metadata.json", path, prefix)).exists() {
            format!("{}/{}/metadata.json", path, prefix)
        } else {
            format!("{}/{}/index.json", path, prefix)
        };
        let tilejson_path = Path::new(&tilejson_path);


        if ! tilejson_path.exists() {
            println!("Need to download {}", tilejson_url);
            match download_url_and_save_to_file(&tilejson_url, tilejson_path) {
                Ok(_) => {
                    println!("Downloaded tilejson for prefix {}, saved to {:?}", prefix, tilejson_path)
                },
                Err(e) => {
                    println!("Error downloading the tilejson for prefix {}, Error was: {:?}.\nExiting", prefix, e);
                    ::std::process::exit(1);
                }
            }
        }
    }
}

fn tilejson_contents(path: &str, urlprefix: &str, pathprefix: &URLPathPrefix, maxzoom: u8) -> Result<String, IompairTileJsonError> {
    // FIXME Remove the unwraps and replace with proper error handling
    let new_tiles = json::Json::from_str(&format!("[\"{}{}{{z}}/{{x}}/{{y}}.pbf\"]", urlprefix, pathprefix.path_with_trailing_slash())).unwrap();
    let zoom_element = json::Json::U64(maxzoom as u64);

    let sub_paths = pathprefix.paths(path);

    let mut tilejson_contents = Vec::with_capacity(sub_paths.len());

    // Collect all the existing tilejsons
    for directory in sub_paths {

        // FIXME don't fall over if there is no file
        // TODO do proper std::path stuff here, instead of string concat
        let tilejson_path = if Path::new(&format!("{}/index.json", directory)).exists() {
            format!("{}/index.json", directory)
        } else {
            format!("{}/metadata.json", directory)
        };

        let mut f = try!(File::open(tilejson_path).map_err(IompairTileJsonError::OpenFileError));
        let mut s = String::new();
        try!(f.read_to_string(&mut s).map_err(IompairTileJsonError::ReadFileError));

        // Some back and forth to decode, replace and encode to get the new tilejson string
        let tilejson_0 = try!(json::Json::from_str(&s).map_err(IompairTileJsonError::InvalidJsonError));
        let mut tilejson = try!(tilejson_0.as_object().ok_or(IompairTileJsonError::NoJSONObjectError)).to_owned();
        tilejson.insert("tiles".to_owned(), new_tiles.clone());
        tilejson.insert("maxzoom".to_owned(), zoom_element.clone());
        tilejson_contents.push(tilejson);
    }

    // now create a new one with the merged vector_layers attribute
    // Copy the first one as base.
    let mut tilejson_base = tilejson_contents.remove(0);
    for mut tilejson in tilejson_contents.into_iter() {
        let mut vector_layers = tilejson.remove("vector_layers").unwrap();
        let mut vector_layers = vector_layers.as_array_mut().unwrap();
        tilejson_base.get_mut("vector_layers").unwrap().as_array_mut().unwrap().append(vector_layers);
    }

    let new_tilejson_contents: String = try!(json::encode(&tilejson_base).map_err(IompairTileJsonError::JsonEncoderError));
    Ok(new_tilejson_contents)
}

fn base_handler(req: Request, mut res: Response, path_format: DirectoryLayout, path: &str, maxzoom: u8, urlprefix: &str, verbose: bool, upstreams: &HashMap<String, String>) {
    let mut url: String = String::new();
    if let hyper::uri::RequestUri::AbsolutePath(ref u) = req.uri {
        url = u.clone();
    }
        
    match parse_url(&url, maxzoom) {
        URL::Tilejson(pathprefix) => {
            tilejson_handler(res, path, urlprefix, &pathprefix, maxzoom);
            if verbose {
                println!("{}/index.json", pathprefix);
            }
        },
        URL::Invalid => {
            *res.status_mut() = hyper::status::StatusCode::NotFound;
        },
        URL::Tile(pathprefix, z, x, y, ext) => {
            tile_handler(res, path_format, path, &pathprefix, z, x, y, ext, &upstreams);
            if verbose {
                println!("{}/{}/{}/{}.pbf", pathprefix, z, x, y);
            }
        }
    }
}

fn tile_handler(mut res: Response, path_format: DirectoryLayout, path: &str, pathprefix: &URLPathPrefix, z: u8, x: u32, y: u32, ext: String, upstreams: &HashMap<String, String>) {
    let tile = Tile::new(z, x, y);
    let tile = try_or_err!(tile.ok_or("ERR"), res, format!("Error when turning z {} x {} y {} into tileobject", z, x, y));

    let mut vector_tiles: Vec<Vec<u8>> = Vec::with_capacity(pathprefix.len());

    for prefix in pathprefix.parts() {

        let sub_path = format!("{}/{}", path, prefix);
        let path = format!("{}/{}", sub_path, match path_format {
            DirectoryLayout::TCPath => tile.tc_path(&ext),
            DirectoryLayout::TSPath => tile.ts_path(&ext),
            DirectoryLayout::ZXYPath => tile.zxy_path(&ext),
        });
        let this_tile_path = Path::new(&path);

        // This is a stupid bit of hackery to ensure that s is initialised to /something/
        let mut this_vector_tile_contents: Vec<u8> = Vec::new();
    
        if this_tile_path.exists() {
            let mut file = try_or_err!(fs::File::open(this_tile_path), res, format!("Error when opening file {:?}", this_tile_path));
            try_or_err!(file.read_to_end(&mut this_vector_tile_contents), res, format!("Error when trying to send vectortile contents to client"));
        } else {
            // File not found, look at our upstream sources if this prefix exists (which also
            // handles cases where /no/ upstreams have been specified)
            // If we don't have any upstream sources for this prefix, then we return (and save)
            // nothing.
            // TODO are there too many print statements here?
            if let Some(upstream_prefix) = upstreams.get(&prefix) {
                let upstream_url = format!("{}/{}/{}/{}.pbf", upstream_prefix, z, x, y);
                println!("Cache miss {}/{}/{}/{}, downloading... ", prefix, z, x, y);

                match download_url(&upstream_url, 10) {
                    Err(e) => {
                        println!("Cache miss {}/{}/{}/{} and error downloading file: {:?}", prefix, z, x, y, e);
                        *res.status_mut() = hyper::status::StatusCode::InternalServerError;
                        return;
                    }
                    Ok(mut new_bytes) => {
                        this_vector_tile_contents.append(&mut new_bytes);
                        match save_to_file(this_tile_path, &this_vector_tile_contents) {
                            Ok(_) => { println!("Cache miss {}/{}/{}/{} downloaded and saved in {:?}", prefix, z, x, y, this_tile_path); }
                            Err(e) => {
                                println!("Cache miss {}/{}/{}/{} and error downloading file: {:?}", prefix, z, x, y, e);
                                *res.status_mut() = hyper::status::StatusCode::InternalServerError;
                                return;
                            }
                        }
                    }
                }
            }
        }

        vector_tiles.push(this_vector_tile_contents);
    }

    let vector_tile = merge_vector_tiles(vector_tiles);

    *res.status_mut() = hyper::status::StatusCode::Ok;
    res.headers_mut().set(ContentType(Mime(TopLevel::Application, SubLevel::Ext("x-protobuf".to_owned()), vec![])));

    // FIXME Cache headers? This says "no caching", which is probably not what's wanted
    //res.headers_mut().set(CacheControl(vec![CacheDirective::Private, CacheDirective::NoCache, CacheDirective::MaxAge(0)]));

    res.send(&vector_tile).unwrap_or_else(|e| {
        println!("Error when trying to send tilejson to client: {:?}", e);
    });

}

fn tilejson_handler(mut res: Response, path: &str, urlprefix: &str, pathprefix: &URLPathPrefix, maxzoom: u8) {
    match tilejson_contents(path, &urlprefix, pathprefix, maxzoom) {
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

fn parse_out_upstreams(args: Option<clap::Values>) -> HashMap<String, String> {
    let mut upstreams: HashMap<String, String> = HashMap::new();
    if let Some(raw) = args {
        let raw: Vec<_> = raw.collect();
        if raw.len() > 0 {
            let mut i = 0;
            loop {
                // this is like the past
                // currently unstable step_by on range will help
                if i >= raw.len() { break; }
                upstreams.insert(raw[i].to_string(), raw[i+1].to_string());
                i += 2;
            }
        }
    }
    upstreams
}
