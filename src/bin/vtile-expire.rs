extern crate clap;
extern crate hyper;
extern crate slippy_map_tiles;
extern crate simple_parallel;
extern crate iter_progress;

use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::unix::fs::MetadataExt;
use std::os::unix::raw::time_t;
use std::thread::sleep_ms;

use clap::{Arg, App};

use hyper::Client;
use slippy_map_tiles::{Tile, BBox};
use iter_progress::ProgressableIter;

fn dl_tile_if_older(tile: Tile, tc_path: &str, upstream_url: &str, expiry_mtime: time_t) {
    let x = tile.x();
    let y = tile.y();
    let z = tile.zoom();

    let path = format!("{}/{}", tc_path, tile.tc_path("pbf"));
    let this_tile_tc_path = Path::new(&path);

    let should_dl = if ! this_tile_tc_path.exists() {
            true
        } else {
            let mtime = this_tile_tc_path.metadata().unwrap().mtime();
            mtime < expiry_mtime
        };

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


fn main() {

    let options = App::new("vtiles-expire")
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
        .arg(Arg::with_name("expire_path").short("e").long("expire-path")
             .takes_value(true).required(true)
             .help("Directory which stores the expire-*.txt files").value_name("PATH"))
        .get_matches();

    let upstream_url = options.value_of("upstream_url").unwrap().to_string();
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    let threads = options.value_of("threads").unwrap_or("4").parse().unwrap();

    let expire_path = options.value_of("expire_path").unwrap().to_string();


    println!("Starting {} threads", threads);
    let mut pool = simple_parallel::Pool::new(threads);

    let expire_directory = Path::new(&expire_path);



    loop {

        let expire_filenames: Vec<_> = expire_directory.read_dir().unwrap().filter_map(|entry| { entry.ok() }).filter(|entry| { let is_file = entry.file_type().unwrap().is_file() ; let file_name = entry.path().file_name().unwrap().to_str().unwrap().to_string(); is_file && file_name.starts_with("expire-") && file_name.ends_with(".txt") }).map(|entry| { entry.path() }).collect();

        if expire_filenames.len() == 0 {
            // Nothing to do, sleeping for 1 minute
            println!("Nothing to do");
            sleep_ms(60 * 1000);
            continue;
        }

        println!("Found {} files ({:?}) to process", expire_filenames.len(), expire_filenames);

        for filename_path in expire_filenames {
            let filename = filename_path.file_name().unwrap().to_str().unwrap();
            let lines: Vec<_> = BufReader::new(fs::File::open(&filename_path).unwrap()).lines().filter_map(|l| { l.ok() }).collect();
            println!("Processing {:?} which has {} lines", filename, lines.len());

            // Could use a .filter_map to get the from_tms, but we presume all the input is OK. Using a
            // map ensures that we have accurate sizes which means the iter-progress can give us
            // percentage views
            let tiles = lines.iter().map(|l| { Tile::from_tms(l.as_str()).unwrap() });

            let expiry_mtime = filename_path.metadata().unwrap().mtime();

            pool.for_(tiles.progress(), |(state, tile)| {
                state.print_every(100, format!("{:.0}% done ({:.1}/sec), tile {:?}       \r", state.percent().unwrap(), state.rate(), tile));
                dl_tile_if_older(tile, &tc_path, &upstream_url, expiry_mtime);
            });
            println!("\nFinished processing file {:?}", filename);
            fs::rename(&filename_path, &filename_path.parent().unwrap().join(format!("done-{}", filename))).unwrap();
        }
    }

    print!("\n");
}

