extern crate clap;
extern crate hyper;
extern crate slippy_map_tiles;
extern crate simple_parallel;
extern crate iter_progress;

use std::io::Read;
use std::fs;
use std::path::Path;
use std::io::Write;

use clap::{Arg, App};

use hyper::Client;
use slippy_map_tiles::{Tile, BBox};
use iter_progress::ProgressableIter;

fn dl_tile(tile: Tile, tc_path: &str, upstream_url: &str) {
    let x = tile.x();
    let y = tile.y();
    let z = tile.zoom();

    let path = format!("{}/{}", tc_path, tile.tc_path("pbf"));
    let this_tile_tc_path = Path::new(&path);

    if ! this_tile_tc_path.exists() {
        let client = Client::new();
        let mut result = client.get(&format!("{}/{}/{}/{}.pbf", upstream_url, z, x, y)).send();
        if result.is_err() { return; }
        let mut result = result.unwrap();
        if result.status != hyper::status::StatusCode::Ok {
            return;
        }

        let mut vector_tile_contents: Vec<u8> = Vec::new();
        result.read_to_end(&mut vector_tile_contents);

        let parent_directory = this_tile_tc_path.parent();
        if parent_directory.is_none() { return; }
        let parent_directory = parent_directory.unwrap();
        if ! parent_directory.exists() {
            fs::create_dir_all(parent_directory);
        }

        let mut file = fs::File::create(this_tile_tc_path);
        if file.is_err() { return; }
        let mut file = file.unwrap();
        file.write_all(&vector_tile_contents);
    }

}

fn dl_tilejson(tc_path: &str, upstream_url: &str) {
    let client = Client::new();
    let mut result = client.get(&format!("{}/index.json", upstream_url)).send();
    if result.is_err() { return; }
    let mut result = result.unwrap();
    if result.status != hyper::status::StatusCode::Ok {
        return;
    }

    let mut contents: Vec<u8> = Vec::new();
    result.read_to_end(&mut contents);

    let path = format!("{}/index.json", tc_path);
    let path = Path::new(&path);
    let parent_directory = path.parent();
    if parent_directory.is_none() { return; }
    let parent_directory = parent_directory.unwrap();
    if ! parent_directory.exists() {
        fs::create_dir_all(parent_directory);
    }

    let mut file = fs::File::create(path);
    if file.is_err() { return; }
    let mut file = file.unwrap();
    file.write_all(&contents);

}


fn main() {

    // FIXME the upstream URL should be changed to take a tilejson URL

   let options = App::new("vtiles-stuffer")
        .setting(clap::AppSettings::AllowLeadingHyphen)
        .arg(Arg::with_name("upstream_url").short("u").long("upstream")
             .takes_value(true).required(true)
             .help("URL of the upstream vector tiles producer").value_name("URL"))
        .arg(Arg::with_name("tc_path").short("c").long("tc-path")
             .takes_value(true).required(true)
             .help("Directory to use as a tile cache.").value_name("PATH"))
        .arg(Arg::with_name("threads").short("T").long("threads")
             .takes_value(true).required(false)
             .help("Number of threads").value_name("THREADS"))
        .arg(Arg::with_name("max-zoom").short("z").long("max-zoom")
             .takes_value(true).required(false)
             .help("Maximum zoom to go to").value_name("ZOOM"))
        .arg(Arg::with_name("top").short("t").long("top")
             .takes_value(true).required(false))
        .arg(Arg::with_name("left").short("l").long("left")
             .takes_value(true).required(false))
        .arg(Arg::with_name("bottom").short("b").long("bottom")
             .takes_value(true).required(false))
        .arg(Arg::with_name("right").short("r").long("right")
             .takes_value(true).required(false))
        .get_matches();

    let upstream_url = options.value_of("upstream_url").unwrap().to_string();
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    let threads = options.value_of("threads").unwrap_or("4").parse().unwrap();
    let max_zoom = options.value_of("max-zoom").unwrap_or("14").parse().unwrap();

    let top = options.value_of("top").unwrap_or("90").parse().unwrap();
    let bottom = options.value_of("bottom").unwrap_or("-90").parse().unwrap();
    let left = options.value_of("left").unwrap_or("-180").parse().unwrap();
    let right = options.value_of("right").unwrap_or("180").parse().unwrap();

    // Download the tilejson file and save it for later.
    dl_tilejson(&tc_path, &upstream_url);
    println!("Downloaded TileJSON");

    println!("Starting {} threads", threads);
    let mut pool = simple_parallel::Pool::new(threads);

    // FIXME unfortunate duplicate with the pool.for_ line

    if top == 90. && bottom == -90. && left == -180. && right == 180. {
        // We're doing the whole world
        let iter = Box::new(Tile::all_to_zoom(max_zoom));
        pool.for_(iter.progress(), |(state, tile)| {
            state.print_every(100, format!("{} done ({}/sec), tile {:?}       \r", state.num_done(), state.rate(), tile));
            dl_tile(tile, &tc_path, &upstream_url);
        });
    } else {
        match slippy_map_tiles::BBox::new(top, left, bottom, right) {
            None => {
                println!("Invalid bbox");
                return;
            },
            Some(b) => {
                let iter = b.tiles().take_while(|&t| { t.zoom() <= max_zoom });

                pool.for_(iter.progress(), |(state, tile)| {
                    state.print_every(100, format!("{} done ({}/sec), tile {:?}       \r", state.num_done(), state.rate(), tile));
                    dl_tile(tile, &tc_path, &upstream_url);
                });
            },
        }
    }

    print!("\n");
}
