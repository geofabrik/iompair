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
use slippy_map_tiles::Tile;
use iter_progress::ProgressableIter;

fn dl_tile(tile: Tile, tc_path: &str, upstream_url: &str) {
    let x = tile.x;
    let y = tile.y;
    let z = tile.zoom;


    // FIXME do not save if it's an error
    let path = format!("{}/{}", tc_path, tile.tc_path("pbf"));
    let this_tile_tc_path = Path::new(&path);

    if ! this_tile_tc_path.exists() {
        let client = Client::new();
        let mut result = client.get(&format!("{}/{}/{}/{}.pbf", upstream_url, z, x, y)).send();
        if result.is_err() { return; }
        let mut result = result.unwrap();

        let mut vector_tile_contents: Vec<u8> = Vec::new();
        // used to have an unwrap here, but that panic'ed
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

    let options = App::new("vtiles-stuffer")
        .arg(Arg::with_name("upstream_url").short("u").long("upstream")
             .takes_value(true).required(true)
             .help("URL of the upstream vector tiles producer").value_name("URL"))
        .arg(Arg::with_name("tc_path").short("c").long("tc-path")
             .takes_value(true).required(true)
             .help("Directory to use as a tile cache.").value_name("PATH"))
        .arg(Arg::with_name("threads").short("t").long("threads")
             .takes_value(true).required(false)
             .help("Number of threads").value_name("THREADS"))
        .get_matches();

    let upstream_url = options.value_of("upstream_url").unwrap().to_string();
    let tc_path = options.value_of("tc_path").unwrap().to_string();
    let threads = options.value_of("threads").unwrap_or("4").parse().unwrap();


    println!("Starting {} threads", threads);
    let mut pool = simple_parallel::Pool::new(threads);
    
    pool.for_(Tile::all_to_zoom(14).progress(), |(state, tile)| {
            state.print_every(100, format!("{} done ({}/sec), tile {:?}       \r", state.num_done(), state.rate(), tile));
            dl_tile(tile, &tc_path, &upstream_url);
    });

    print!("\n");
}
