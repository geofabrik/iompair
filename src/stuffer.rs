extern crate clap;
extern crate hyper;
extern crate slippy_map_tiles;
extern crate simple_parallel;
extern crate iter_progress;
extern crate chrono;

use std::path::Path;
use std::os::unix::fs::MetadataExt;

use clap::ArgMatches;
use slippy_map_tiles::Tile;
use iter_progress::ProgressableIter;
use chrono::{DateTime, FixedOffset};

use utils::{download_url_and_save_to_file, IompairError, DirectoryLayout};

fn dl_tile(tile: Tile, path: &str, path_format: DirectoryLayout, upstream_url: &str, always_download: bool, files_older_than: &Option<DateTime<FixedOffset>>) -> Result<(), IompairError> {
    let x = tile.x();
    let y = tile.y();
    let z = tile.zoom();

    let this_path = format!("{}/{}", path, match path_format {
        DirectoryLayout::TCPath => tile.tc_path("pbf"),
        DirectoryLayout::TSPath => tile.ts_path("pbf"),
        DirectoryLayout::ZXYPath => tile.zxy_path("pbf"),
    });
    let this_path = Path::new(&this_path);

    let should_download = if ! this_path.exists() {
        true
    } else {
        if always_download {
            match *files_older_than {
                None => { true },
                Some(dt) => {
                    let mtime = this_path.metadata().unwrap().mtime();
                    let cutoff = dt.timestamp();
                    mtime < cutoff
                }
            }
        } else {
            false
        }
    };

    if should_download {
        try!(download_url_and_save_to_file(&format!("{}/{}/{}/{}.pbf", upstream_url, z, x, y), this_path));
    }

    Ok(())
}

fn dl_tilejson(path: &str, upstream_url: &str) -> Result<(), IompairError> {
    try!(download_url_and_save_to_file(&format!("{}/index.json", upstream_url), Path::new(&format!("{}/index.json", path))));
    Ok(())
}


pub fn stuffer(options: &ArgMatches) {

    let upstream_url = options.value_of("upstream_url").unwrap().to_string();
    let path = options.value_of("tc_path").or(options.value_of("ts_path")).or(options.value_of("zxy_path")).unwrap().to_string();
    let path_format = if options.is_present("tc_path") { DirectoryLayout::TCPath } else if options.is_present("ts_path") { DirectoryLayout::TSPath } else if options.is_present("zxy_path") { DirectoryLayout::ZXYPath } else { unreachable!() };
    let threads = options.value_of("threads").unwrap().parse().unwrap();
    let max_zoom = options.value_of("max-zoom").unwrap().parse().unwrap();
    let min_zoom: u8 = options.value_of("min-zoom").unwrap().parse().unwrap();

    let always_download = options.is_present("always-download");
    let files_older_than: Option<DateTime<FixedOffset>> = options.value_of("files-older-than").and_then(|t| { DateTime::parse_from_rfc3339(t).ok() });

    let top = options.value_of("top").unwrap().parse().unwrap();
    let bottom = options.value_of("bottom").unwrap().parse().unwrap();
    let left = options.value_of("left").unwrap().parse().unwrap();
    let right = options.value_of("right").unwrap().parse().unwrap();


    // Download the tilejson file and save it for later.
    dl_tilejson(&path, &upstream_url).unwrap_or_else(|e| {
        println!("Error occured when downloading tilejson: {:?}", e);
        println!("Aborting");
        return;
    });
    println!("Downloaded TileJSON");

    println!("Starting {} threads", threads);
    let mut pool = simple_parallel::Pool::new(threads);

    // FIXME unfortunate duplicate with the pool.for_ line

    if top == 90. && bottom == -90. && left == -180. && right == 180. {
        // We're doing the whole world
        let iter = Box::new(Tile::all_to_zoom(max_zoom).filter(|&t| { t.zoom() >= min_zoom }));
        pool.for_(iter.progress(), |(state, tile)| {
            state.print_every_n_sec(5., format!("{} done ({}/sec), tile {:?}       \r", state.num_done(), state.rate(), tile));
            dl_tile(tile, &path, path_format, &upstream_url, always_download, &files_older_than).unwrap_or_else(|e| {
                println!("Error occured when downloading tile {:?}: {:?}", tile, e);
            });
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
                    state.print_every_n_sec(1., format!("{} done ({}/sec), tile {:?}       \r", state.num_done(), state.rate(), tile));
                    //state.print_every_sec(100., format!("{} done ({}/sec), tile {:?}       \r", state.num_done(), state.rate(), tile));
                    dl_tile(tile, &path, path_format, &upstream_url, always_download, &files_older_than).unwrap_or_else(|e| {
                        println!("Error occured when downloading tile {:?}: {:?}", tile, e);
                    });
                });
            },
        }
    }

    print!("\n");
}
