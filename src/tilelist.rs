extern crate slippy_map_tiles;

use std::path::Path;
use clap::ArgMatches;
use slippy_map_tiles::Tile;

pub fn tilelist(options: &ArgMatches) {
    let max_zoom; let min_zoom;
    if options.is_present("zoom") {
        let zoom: u8 = options.value_of("zoom").unwrap().parse().unwrap();
        min_zoom = zoom;
        max_zoom = zoom;
    } else {
        min_zoom = options.value_of("min-zoom").unwrap_or("0").parse().unwrap();
        max_zoom = options.value_of("max-zoom").unwrap_or("14").parse().unwrap();
    }
    let not_exists = options.is_present("not_exists");
    let root_dir = options.value_of("ts_path");

    // FIXME iterating over all zoom levels less than min-zoom is probably a bit ineffecient, but
    // it's simple and it works

    for tile in Tile::all().take_while(|t| t.zoom() <= max_zoom).filter(|t| t.zoom() >= min_zoom) {
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
