extern crate slippy_map_tiles;

use std::path::Path;
use clap::ArgMatches;
use slippy_map_tiles::Tile;

pub fn tilelist(options: &ArgMatches) {
    let max_zoom: u8 = options.value_of("max-zoom").unwrap().parse().unwrap();
    let not_exists = options.is_present("not_exists");
    let root_dir = options.value_of("ts_path");

    for tile in Tile::all().take_while(|t| t.zoom() <= max_zoom) {
        let include = if not_exists {
            let tile_path = format!("{}/{}", root_dir.unwrap(), tile.ts_path("pbf"));
            let tile_path = Path::new(&tile_path);
            ! tile_path.exists()
        } else {
            // include everything
            true
        };

        if include {
            println!("{}/{}/{}", tile.zoom(), tile.x(), tile.y());
        }

    }
}
