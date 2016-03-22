extern crate iron;
extern crate router;
extern crate hyper;
extern crate clap;
extern crate rustc_serialize;
extern crate slippy_map_tiles;
extern crate regex;
extern crate tilejson;

use clap::{Arg, App, ArgMatches, SubCommand};

mod cache;
mod serve;

use cache::cache;
use serve::serve;


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
        .get_matches();

    match options.subcommand() {
        ("cache", Some(options)) => { cache(options); },
        ("serve", Some(options)) => { serve(options); },
        (_, _) => { println!("{}", options.usage()); },
    }

}
