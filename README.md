# iompair

Iompair (from [the Irish for to transmit, to vector](https://en.wiktionary.org/wiki/iompair#Irish)) is a collection of
programmes to help working with [vectortiles](https://wiki.openstreetmap.org/wiki/Vector_tiles). It allows you to store pbf vector
tiles in a simple directory hiearchy, and serve them over HTTP.

You can `serve` vector tiles from a directory over TMS URLs, use the `stuffer`
to fill you local cache from an upstream vector tile cache, and use `expire` to
expire tiles from an `osm2pgsql` tile expire list.

# Compiling

    cargo build

This will build a debug build. `cargo build --release` will build a "release"
build, which takes longer to compile, but will perform optimizations and a
smaller binary. It's unlike that iompair will be CPU limited.

# Usage

## iompair serve

Serves a TileCache directory over HTTP over a port.

    iompair serve --port 9000 --tc-path /path/to/files/ --urlprefix http://example.com/mytiles/

It also supports TileStash (`--ts-path`), and ZXY directory layouts
(`--zxy-path`) (where files are stored `/path/to/file/zoom/X/Y.pbf`)

### Directory Layouts

#### TileCache

A TileCache directory is of the form `Z/XXX/XXX/XXX/YYY/YYY/YYY.pbf`

#### TileStash

A TileStash directory is of the form `Z/XXX/XXX/YYY/YYY.pbf`

#### ZYX

No splitting, files are stored in format `Z/X/Y.pbf` (e.g. `mb-util` will
create this)

### TileJSON URLs

The [TileJSON](https://github.com/mapbox/tilejson-spec) url is `/index.json`.
It is the contents of the file `index.json` in the root of the
`--tc/ts/zxy-path` (or `metadata.json` if that doesn't exist.

### Merging multiple vector tiles together

`iompair serve` can serve just one set of vector tiles, or multiple sets.

If you have `--zxy-path /data/tiles/` and the directories `/data/tiles/land/`,
`/data/tiles/points/` and `/data/tiles/roads/` with each directory having a
separate set of vectortiles, then you have 3 prefixes for 3 tilesets (`land`,
`points`, `roads`), and:

 * A request to `/0/0/0.pbf` won't work
 * `/land/0/0/0.pbf` or `/land/index.json` will return the tiles from the
   `land` directory (etc)
 * Double underscore (`__`) can be used to concatinate prefixes.
   `/land__points/0/0/0.pbf` will return the `0/0/0.pbf` from `land` directory,
   and then the `0/0/0.pbf` form `points` will be concatinated on afterwards.
   (`/points__land/...` will have them in the other order). This can be
   extended many times. `/land__points__roads/0/0/0.pbf` will be `land`, then
   `points`, the `roads`.

#### TileJSON

Each concatinated tileset will have a TileJSON showing the layers in the
subparts. `/land__points__roads/index.json` etc. The TileJSON will be the
correctly concatinated JSON of the sub tilesets

### Fetching from upstream

If the `--upstream` argument is given, and a tile is requested which doesn't
exist, then `iompair` will request it from this upstream URL. This flag takes 2
arguments, a `PREFIX` and the corresponding `URL` for the upstream for this PREFIX.

Example:

    iompair serve --port 9000 --zxy-path /data/tiles --upstream land http://example.com/landtiles/ --upstream points http://localhost:8080/

When a `land` tile doesn't exist in `/data/tiles/land/`, it will be downloaded
from the URL `http://example.com/landtiles/$ZOOM/$X/$Y.pbf`, saved locally to
the land directory, and then served up to the client. Likewise for `points`.
TileJSON for the upstream URLs is not supported.

### Post Fetch Command

If you use `--upstream`, you can also specify `--post-fetch-command` (which
takes one argument) after a file is downloaded from upstream, this command will
be executed with the local filename of the newly downloaded tile as the first
and only argument.

You can use this to copy a newly downloaded tile to other servers, or do
whatever you want really.

Example:

    iompair serve --port 9000 --zxy-path /data/tiles --upstream land http://example.com/landtiles/ --upstream points http://localhost:8080/ --post-fetch-command /usr/local/bin/copy_tiles.sh

## iompair expire

Reads all the expire filename in a directory, looking for files, parses out the
TMS tile references and updates the files stored in the vector tile directory
from the upstream, pbf vector tile source.

It will not finish, but wait until there are new files available, sleeping
between runs

    iompair expire --tc-path /path/to/vector/tile/store --upstream http://example.com/tiles/ --expire-path /path/to/osm2pgsql/expired-tiles/

## iompair stuffer

Populates (stuffs) a tilecache laidout directory with tiles from an upstream
vector tile source. Specifiy the number of threads with `-T`.

    iompair stuffer --tc-path /path/to/put/vector/tiles --upstream http://example.com/tiles/ -z 14 -b 35.55 -t 71.6 -l -25.93 -r 48.95 -T 20

# Copyright & Licence

Copyright 2016 Geofabrik GmbH, licenced under the GNU General Public Licence
version 3 (or later).
