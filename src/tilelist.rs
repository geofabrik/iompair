extern crate slippy_map_tiles;

use std::path::Path;
use clap::ArgMatches;
use slippy_map_tiles::Tile;

pub fn tilelist(options: &ArgMatches) {
    let min_zoom: u32 = options.value_of("min-zoom").unwrap().parse().unwrap();
    let max_zoom: u32 = options.value_of("max-zoom").unwrap().parse().unwrap();
    let not_exists = options.is_present("not_exists");
    let root_dir = options.value_of("ts_path");

    for zoom in min_zoom..(max_zoom+1) {
        // TODO there's probably ways to use iterators here
        let mut tiles: Vec<(u32, u32)> = Vec::with_capacity(2 * 2usize.pow(zoom));
        for x in 0..2u32.pow(zoom) {
            for y in 0..2u32.pow(zoom) {
                let include = if not_exists {
                    let tile = Tile::new(zoom as u8, x, y).unwrap();
                    let tile_path = format!("{}/{}", root_dir.unwrap(), tile.ts_path("pbf"));
                    let tile_path = Path::new(&tile_path);
                    ! tile_path.exists()
                } else {
                    // include everything
                    true
                };
                if include {
                    tiles.push((x, y));
                }
            }
        }
        tiles.shrink_to_fit();

        tiles.sort_by_key(|&(x, y)| z_order_key(x, y));
        for (x, y) in tiles {
            println!("{}/{}/{}", zoom, x, y);
        }

    }
}

fn z_order_key(x: u32, y: u32) -> u64 {
    // FIXME there is probably a better way to do this where I look at if it's divisible by powers
    // or 2, rather than creating strings
    let x_binary: Vec<char> = format!("{:032b}", x).chars().collect();
    let y_binary: Vec<char> = format!("{:032b}", y).chars().collect();
    let mut new_string = String::with_capacity(64);
    for i in 0..32 {
        new_string.push(x_binary[i]);
        new_string.push(y_binary[i]);
    }

    let res = u64::from_str_radix(&new_string, 2).unwrap();

    res
}

mod test {
    #[test]
    fn test_zorder() {
        use super::z_order_key;
        assert_eq!(z_order_key(0, 0), 0);
        assert_eq!(z_order_key(0, 1), 1);
        assert_eq!(z_order_key(1, 0), 2);
        assert_eq!(z_order_key(1, 1), 3);

    }

}
