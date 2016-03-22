<a name="v0.2.0"></a>
## v0.2.0 (2016-03-22)


#### Bug Fixes

*   Support current slippy-map-tiles API ([4569777e](4569777e))
*   Sleep better ([9a23cb98](9a23cb98))
*   Only save the file if there is a 200 status ([2fec2a71](2fec2a71))

#### Features

*   Add vtile-expire which refreshes tile that expired ([3aa5bdf7](3aa5bdf7))
*   Add -z/--max-zoom option to control TileJSON output ([aaa814a1](aaa814a1))
*   vtile-stuffer can now take bounding box ([a54a4bfb](a54a4bfb))
*   TileJSON output now calculated dynamically ([a32e8800](a32e8800))
* **cache:**  Make a subcommand ([3de4ba5c](3de4ba5c))
* **expire:**  Now a subcommand ([84ee0c4a](84ee0c4a))
* **serve:**
  *  Now a subcommand ([5a31ccb4](5a31ccb4))
  *  Serve up locally saved tilejson ([46f6bb4c](46f6bb4c))
* **stuffer:**
  *  Now a subcommand ([23834d6f](23834d6f))
  *  Add --always-download option ([c0be16fd](c0be16fd))
  *  Add --min-zoom arg - skip some zoom levels ([629622b8](629622b8))
  *  Download the upstream tilejson and save ([4d42dec8](4d42dec8))



<a name=""></a>
##  (2016-03-09)


#### Features

*   Add -z/--max-zoom option (default 14) ([f49be184](f49be184))



