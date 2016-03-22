extern crate clap;
extern crate hyper;
extern crate slippy_map_tiles;
extern crate simple_parallel;
extern crate iter_progress;

use std::io::Read;
use std::fs;
use std::path::Path;
use std::io::Write;

use clap::{Arg, App, ArgMatches};

use hyper::Client;
use slippy_map_tiles::{Tile, BBox};
use iter_progress::ProgressableIter;

fn dl_tile(tile: Tile, tc_path: &str, upstream_url: &str, always_download: bool) {
    let x = tile.x();
    let y = tile.y();
    let z = tile.zoom();

    // FIXME replace with proper path opts
    let path = format!("{}/{}", tc_path, tile.tc_path("pbf"));
    let this_tile_tc_path = Path::new(&path);

    if ! this_tile_tc_path.exists() || always_download {
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


pub fn stuffer(options: &ArgMatches) {

    let upstream_url = options.value_of("upstream_url").unwrap().to_string();
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    let threads = options.value_of("threads").unwrap().parse().unwrap();
    let max_zoom = options.value_of("max-zoom").unwrap().parse().unwrap();
    let min_zoom: u8 = options.value_of("min-zoom").unwrap().parse().unwrap();

    let always_download = options.is_present("always-download");

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
        let iter = Box::new(Tile::all_to_zoom(max_zoom).filter(|&t| { t.zoom() >= min_zoom }));
        pool.for_(iter.progress(), |(state, tile)| {
            state.print_every(100, format!("{} done ({}/sec), tile {:?}       \r", state.num_done(), state.rate(), tile));
            dl_tile(tile, &tc_path, &upstream_url, always_download);
        });
    } else {
        match slippy_map_tiles::BBox::new(top, left, bottom, right) {
            None => {
                println!("Invalid bbox");
                return;
            },
            Some(b) => {
                let iter = b.tiles().filter(|&t| { t.zoom() >= min_zoom }).take_while(|&t| { t.zoom() <= max_zoom });

                pool.for_(iter.progress(), |(state, tile)| {
                    state.print_every(100, format!("{} done ({}/sec), tile {:?}       \r", state.num_done(), state.rate(), tile));
                    dl_tile(tile, &tc_path, &upstream_url, always_download);
                });
            },
        }
    }

    print!("\n");
}