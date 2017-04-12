extern crate clap;
extern crate hyper;
extern crate slippy_map_tiles;
extern crate simple_parallel;
extern crate iter_progress;

use std::fs;
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::io::prelude::*;
use std::os::unix::fs::MetadataExt;
#[allow(deprecated)]
use std::os::unix::raw::time_t;
use std::thread::sleep;
use std::time::Duration;

use clap::ArgMatches;

use slippy_map_tiles::Tile;
use iter_progress::ProgressableIter;

use utils::download_url_and_save_to_file;

#[allow(deprecated)]
fn dl_tile_if_older(tile: Tile, tc_path: &str, upstream_url: &str, expiry_mtime: time_t) {
    let x = tile.x();
    let y = tile.y();
    let z = tile.zoom();

    let path = format!("{}/{}", tc_path, tile.tc_path("pbf"));
    let this_tile_tc_path = Path::new(&path);

    let should_dl = if ! this_tile_tc_path.exists() {
            true
        } else {
            let mtime = match this_tile_tc_path.metadata() {
                Err(e) => { println!("Error when trying to get tile metadata: {:?}", e); return; },
                Ok(m) => m.mtime(),
            };
            mtime < expiry_mtime
        };

    if should_dl {
        download_url_and_save_to_file(&format!("{}/{}/{}/{}.pbf", upstream_url, z, x, y), this_tile_tc_path).unwrap_or_else(|e| {
            println!("Error occured when downloading {}/{}/{}: {:?}", z, x, y, e);
        });
    }

}

fn get_expire_filenames(expire_directory: &Path) -> Result<Vec<PathBuf>, ()> {
    let entries = try!(expire_directory.read_dir().map_err(|_| ()));
    let entries = entries.filter_map(|entry| { entry.ok() });
    Ok(entries.filter(|entry| {
        let is_file = match entry.file_type().map(|f| f.is_file()) {
            Err(_) => { return false; }, Ok(x) => x };
        let file_name: String = match entry.path().file_name().and_then(|f| f.to_str() ) {
            None => { return false; } Some(x) => x.to_string() };

        is_file && file_name.starts_with("expire-") && file_name.ends_with(".txt")
    }).map(|entry| { entry.path() }).collect::<Vec<_>>())
}

fn single_expire_run(filename_path: &PathBuf, pool: &mut simple_parallel::Pool, tc_path: &str, upstream_url: &str) -> Result<(), String> {
    let filename = try!(try!(filename_path.file_name().ok_or("Couldn't get filename".to_string())).to_str().ok_or("Couldn't convert to string".to_string()));
    let file = try!(fs::File::open(&filename_path).map_err(|_| "Couldnt' open file".to_string()));
    let lines: Vec<_> = BufReader::new(file).lines().filter_map(|l| { l.ok() }).collect();
    println!("Processing {:?} which has {} lines", filename, lines.len());

    // Could use a .filter_map to get the from_tms, but we presume all the input is OK. Using a
    // map ensures that we have accurate sizes which means the iter-progress can give us
    // percentage views
    let tiles = lines.iter().map(|l| { Tile::from_tms(l.as_str()).unwrap() });

    let expiry_mtime = try!(filename_path.metadata().map_err(|_| "Couldn't get metadata".to_string())).mtime();

    pool.for_(tiles.progress(), |(state, tile)| {
        state.print_every_n_items(100, format!("{:.0}% done ({:.1}/sec), tile {:?}       \r", state.percent().map(|x| x.to_string()).unwrap_or("N/A".to_string()), state.rate(), tile));
        dl_tile_if_older(tile, &tc_path, &upstream_url, expiry_mtime);
    });
    let parent_dir = try!(filename_path.parent().ok_or("Directory".to_string()));
    let new_filename = &parent_dir.join(format!("done-{}", filename));


    fs::rename(&filename_path, new_filename).map_err(|_| "Couldn't rename".to_string())
}

pub fn expire(options: &ArgMatches) {

    let upstream_url = options.value_of("upstream_url").unwrap().to_string();
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    let threads = options.value_of("threads").unwrap().parse().unwrap();

    let expire_path = options.value_of("expire_path").unwrap().to_string();

    let wait_between_runs = options.value_of("wait_between_runs").unwrap().parse().unwrap();


    println!("Starting {} threads", threads);
    let mut pool = simple_parallel::Pool::new(threads);

    let expire_directory = Path::new(&expire_path);



    loop {

        let expire_filenames = match get_expire_filenames(expire_directory) {
            Ok(e) => e,
            Err(_) => {
                // Something when wrong trying to get the files
                continue;
            },
        };

        if expire_filenames.len() == 0 {
            // Nothing to do, sleeping
            sleep(Duration::new(wait_between_runs, 0));
            continue;
        }

        println!("Found {} files ({:?}) to process", expire_filenames.len(), expire_filenames);

        for filename_path in expire_filenames {
            match single_expire_run(&filename_path, &mut pool, &tc_path, &upstream_url) {
                Ok(_) => {
                    println!("\nFinished processing file {:?}", filename_path);
                },
                Err(e) => {
                    println!("\nCouldn't download {:?} error: {:?}", filename_path, e);
                },
            }
        }
    }

}

