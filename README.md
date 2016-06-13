# iompair

Iompair (from [the Irish for to transmit, to vector](https://en.wiktionary.org/wiki/iompair#Irish)) is a collection of
programmes to help working with [vectortiles](https://wiki.openstreetmap.org/wiki/Vector_tiles). It allows you to store pbf vector
tiles in a simple directory hiearchy, and serve them over HTTP.

You can `serve` vector tiles from a TileCache layout directory over TMS URLs, use the `stuffer` to fill you local cache from an upstream vector tile cache, use `expire` to expire tiles from an `osm2pgsql` tile expire list, and use `cache` to assist you developing a tile style to prevent you having to regenerate vector tiles from scratch

# Compiling

    cargo build

# Usage

## iompair serve

## iompair expire

## iompair stuffer

## iompair cache

# Copyright & Licence

Copyright 2016 Geofabrik GmbH, licenced under the GNU General Public Licence
version 3 (or later).
