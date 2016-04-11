extern crate clap;
extern crate hyper;
extern crate slippy_map_tiles;
extern crate simple_parallel;
extern crate iter_progress;

use std::fs;
use std::path::Path;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::unix::fs::MetadataExt;
use std::os::unix::raw::time_t;
use std::thread::sleep_ms;

use clap::ArgMatches;

use slippy_map_tiles::Tile;
use iter_progress::ProgressableIter;

use utils::download_url_and_save_to_file;

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
        download_url_and_save_to_file(&format!("{}/{}/{}/{}.pbf", upstream_url, z, x, y), this_tile_tc_path);
    }

}


pub fn expire(options: &ArgMatches) {

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
                state.print_every_n_items(100, format!("{:.0}% done ({:.1}/sec), tile {:?}       \r", state.percent().unwrap(), state.rate(), tile));
                dl_tile_if_older(tile, &tc_path, &upstream_url, expiry_mtime);
            });
            println!("\nFinished processing file {:?}", filename);
            fs::rename(&filename_path, &filename_path.parent().unwrap().join(format!("done-{}", filename))).unwrap();
        }
    }

    print!("\n");
}

