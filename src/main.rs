extern crate hyper;
extern crate clap;
extern crate rustc_serialize;
extern crate slippy_map_tiles;
extern crate regex;
extern crate simple_parallel;
extern crate iter_progress;
extern crate chrono;
extern crate libflate;

use clap::{Arg, App, SubCommand, ArgGroup};


#[macro_use]
mod utils;

mod serve;
mod stuffer;
mod expire;
mod tilelist;

use serve::serve;
use stuffer::stuffer;
use expire::expire;
use tilelist::tilelist;

fn main() {

    let options = App::new("iompair")
        .about("Work with vector tiles")
        .subcommand(SubCommand::with_name("serve")
            .about("Serve a tile cache directory")
            .arg(Arg::with_name("port").short("p").long("port")
                 .takes_value(true).required(true)
                 .help("Port to listen on").value_name("PORT"))
            .arg(Arg::with_name("maxzoom").short("z").long("max-zoom")
                 .takes_value(true).default_value("14")
                 .help("Maximum zoom to pretend").value_name("ZOOM"))
            .arg(Arg::with_name("urlprefix").long("urlprefix")
                 .takes_value(true).required(false)
                 .help("URL that the tiles are accessible under").value_name("URL"))
            .arg(Arg::with_name("tc_path").long("tc-path")
                 .takes_value(true).conflicts_with("ts_path")
                 .help("Directory to use as a tile cache (TileCache layout).").value_name("PATH"))
            .arg(Arg::with_name("ts_path").long("ts-path")
                 .takes_value(true).conflicts_with("tc-path")
                 .help("Directory to use as a tile cache (TileStash safe layout).").value_name("PATH"))
            .arg(Arg::with_name("zxy_path").long("zxy-path")
                 .takes_value(true).conflicts_with("tc-path").conflicts_with("ts-path")
                 .help("Directory to use as a tile cache (ZXY layout).").value_name("PATH"))
            .arg(Arg::with_name("verbose").long("verbose")
                 .takes_value(false)
                 .help("Verbose mode. Prints to stdout at every request served"))
            .group(ArgGroup::with_name("path").args(&["tc_path", "ts_path", "zxy_path"]).required(true))
            )
        .subcommand(SubCommand::with_name("stuffer")
            .about("Populate a tile cache directory with all the tiles in an area")
            .setting(clap::AppSettings::AllowLeadingHyphen)
            .arg(Arg::with_name("upstream_url").short("u").long("upstream")
                 .takes_value(true).required(true)
                 .help("URL of the upstream vector tiles producer").value_name("URL"))
            .arg(Arg::with_name("tc_path").short("c").long("tc-path")
                 .takes_value(true)
                 .help("Directory to use as a tile cache.").value_name("PATH"))
            .arg(Arg::with_name("ts_path").long("ts-path")
                 .takes_value(true).conflicts_with("tc-path")
                 .help("Directory to use as a tile cache (TileStash safe layout).").value_name("PATH"))
            .arg(Arg::with_name("threads").short("T").long("threads")
                 .takes_value(true).required(false).default_value("4")
                 .help("Number of threads").value_name("THREADS"))
            .arg(Arg::with_name("max-zoom").short("z").long("max-zoom")
                 .takes_value(true).required(false).default_value("14")
                 .help("Maximum zoom to go to").value_name("ZOOM"))
            .arg(Arg::with_name("min-zoom").long("min-zoom")
                 .takes_value(true).required(false).default_value("0")
                 .help("Minimum zoom to start from").value_name("ZOOM"))
            .arg(Arg::with_name("top").short("t").long("top")
                 .takes_value(true).required(false).default_value("90"))
            .arg(Arg::with_name("left").short("l").long("left")
                 .takes_value(true).required(false).default_value("-180"))
            .arg(Arg::with_name("bottom").short("b").long("bottom")
                 .takes_value(true).required(false).default_value("-90"))
            .arg(Arg::with_name("right").short("r").long("right")
                 .takes_value(true).required(false).default_value("180"))
            .arg(Arg::with_name("always-download").long("always-download")
                 .takes_value(false).required(false)
                 .help("Always download the files, even if they already exist"))
            .arg(Arg::with_name("files-older-than").long("files-older-than")
                 .takes_value(true).required(false)
                 .help("If using --always-download, only download a file that's missing or older than this RFC3339 datetime"))
            )
        .subcommand(SubCommand::with_name("expire")
            .about("Update a tilecache directory from upstream with osm2pgsql expiry tile list")
            .arg(Arg::with_name("upstream_url").short("u").long("upstream")
                 .takes_value(true).required(true)
                 .help("URL of the upstream vector tiles producer").value_name("URL"))
            .arg(Arg::with_name("tc_path").short("c").long("tc-path")
                 .takes_value(true).required(true)
                 .help("Directory to use as a tile cache.").value_name("PATH"))
            .arg(Arg::with_name("threads").short("T").long("threads")
                 .takes_value(true).required(false).default_value("4")
                 .help("Number of threads").value_name("THREADS"))
            .arg(Arg::with_name("expire_path").short("e").long("expire-path")
                 .takes_value(true).required(true)
                 .help("Directory which stores the expire-*.txt files").value_name("PATH"))
            .arg(Arg::with_name("wait_between_runs").short("w").long("wait")
                 .takes_value(true).required(false).default_value("60")
                 .help("How long (in SEC) to wait between checks of the expire directory. Default 60 sec").value_name("SEC"))
            )
        .subcommand(SubCommand::with_name("tilelist")
            .about("Generate a Z/X/Y tile list (to stdout) based on tiles")
            .arg(Arg::with_name("max-zoom").short("z").long("max-zoom")
                 .takes_value(true).required(false)
                 .help("Maximum zoom to go to").value_name("ZOOM"))
            .arg(Arg::with_name("min-zoom").long("min-zoom")
                 .takes_value(true).required(false)
                 .help("Minimum zoom to start from").value_name("ZOOM"))
            .arg(Arg::with_name("not_exists").long("--not-exists")
                 .takes_value(false).required(false)
                 .help("Only include files which don't exist"))
            .arg(Arg::with_name("zoom").long("zoom")
                 .takes_value(true).required(false)
                 .help("Only generate tiles on this zoom level").value_name("ZOOM")
                 .conflicts_with("min-zoom").conflicts_with("max-zoom")
                 )
            .arg(Arg::with_name("ts_path").long("ts-path")
                 .takes_value(true)
                 .help("Directory to use as a tile cache (TileStash safe layout).").value_name("PATH"))
            )
        .get_matches();

    match options.subcommand() {
        ("serve", Some(options)) => { serve(options); },
        ("stuffer", Some(options)) => { stuffer(options); },
        ("expire", Some(options)) => { expire(options); },
        ("tilelist", Some(options)) => { tilelist(options); },
        (_, _) => { println!("{}", options.usage()); },
    }

}
