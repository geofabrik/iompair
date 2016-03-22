extern crate iron;
extern crate router;
extern crate hyper;
extern crate clap;
extern crate rustc_serialize;
extern crate slippy_map_tiles;
extern crate regex;
extern crate tilejson;
extern crate simple_parallel;
extern crate iter_progress;

use clap::{Arg, App, ArgMatches, SubCommand};

mod cache;
mod serve;
mod stuffer;
mod expire;

use cache::cache;
use serve::serve;
use stuffer::stuffer;
use expire::expire;


fn main() {

    let options = App::new("vtile")
        .subcommand(SubCommand::with_name("cache")
            .arg(Arg::with_name("port").short("p").long("port")
                 .takes_value(true).required(true)
                 .help("Port to listen on").value_name("PORT"))
            .arg(Arg::with_name("upstream_url").short("u").long("upstream")
                 .takes_value(true).required(true)
                 .help("URL of the upstream vector tiles producer").value_name("URL"))
            .arg(Arg::with_name("tc_path").short("c").long("tc-path")
                 .takes_value(true).required(true)
                 .help("Directory to use as a tile cache.").value_name("PATH"))
            )
        .subcommand(SubCommand::with_name("serve")
            .arg(Arg::with_name("port").short("p").long("port")
                 .takes_value(true).required(true)
                 .help("Port to listen on").value_name("PORT"))
            .arg(Arg::with_name("tc_path").short("c").long("tc-path")
                 .takes_value(true).required(true)
                 .help("Directory to use as a tile cache.").value_name("PATH"))
            .arg(Arg::with_name("maxzoom").short("z").long("max-zoom")
                 .takes_value(true)
                 .help("Maximum zoom to preten").value_name("ZOOM"))
            )
        .subcommand(SubCommand::with_name("stuffer")
            .setting(clap::AppSettings::AllowLeadingHyphen)
            .arg(Arg::with_name("upstream_url").short("u").long("upstream")
                 .takes_value(true).required(true)
                 .help("URL of the upstream vector tiles producer").value_name("URL"))
            .arg(Arg::with_name("tc_path").short("c").long("tc-path")
                 .takes_value(true).required(true)
                 .help("Directory to use as a tile cache.").value_name("PATH"))
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
                 .takes_value(true).required(false))
            .arg(Arg::with_name("left").short("l").long("left")
                 .takes_value(true).required(false))
            .arg(Arg::with_name("bottom").short("b").long("bottom")
                 .takes_value(true).required(false))
            .arg(Arg::with_name("right").short("r").long("right")
                 .takes_value(true).required(false))
            .arg(Arg::with_name("always-download").long("always-download")
                 .takes_value(false).required(false)
                 .help("Always download the files, even if they already exist"))
            )
        .subcommand(SubCommand::with_name("expire")
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
            )
        .get_matches();

    match options.subcommand() {
        ("cache", Some(options)) => { cache(options); },
        ("serve", Some(options)) => { serve(options); },
        ("stuffer", Some(options)) => { stuffer(options); },
        ("expire", Some(options)) => { expire(options); },
        (_, _) => { println!("{}", options.usage()); },
    }

}
