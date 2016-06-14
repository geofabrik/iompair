# iompair

Iompair (from [the Irish for to transmit, to vector](https://en.wiktionary.org/wiki/iompair#Irish)) is a collection of
programmes to help working with [vectortiles](https://wiki.openstreetmap.org/wiki/Vector_tiles). It allows you to store pbf vector
tiles in a simple directory hiearchy, and serve them over HTTP.

You can `serve` vector tiles from a TileCache layout directory over TMS URLs, use the `stuffer` to fill you local cache from an upstream vector tile cache, use `expire` to expire tiles from an `osm2pgsql` tile expire list, and use `cache` to assist you developing a tile style to prevent you having to regenerate vector tiles from scratch

# Compiling

    cargo build

# Usage

## iompair serve

Serves a TileCache laidout directory over HTTP over a port.

    iompair serve --port 9000 --tc-path /path/to/files/ --urlprefix http://example.com/mytiles/

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

## iompair cache

It will serve files from a provided directory, and if the file isn't there,
it'll download from the upstream, and cache the file locally. 

# Copyright & Licence

Copyright 2016 Geofabrik GmbH, licenced under the GNU General Public Licence
version 3 (or later).
