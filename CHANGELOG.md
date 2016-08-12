<a name="v0.6.0"></a>
## v0.6.0 (2016-08-12)


#### Features

*   Output when finished ([7f8054a6](7f8054a6))

#### Bug Fixes

*   Work if maxzoom not specified ([e84b0603](e84b0603))
* **serve:**  Return an empty 200, not 404, when tile doesn't exist ([a689ff13](a689ff13))



<a name=""></a>
##  (2016-06-10)


#### Bug Fixes

*   Explicity handle more errors ([996107a3](996107a3))
* **cache:**
  *  Set cache control headers ([7b532820](7b532820))
  *  Remove iron, do it all with hyper now ([b52617f9](b52617f9))
* **expire:**
  *  Logic bug which prevented it expiring tiles ([3c165b88](3c165b88))
  *  Silent output if nothing to do ([2b825389](2b825389))
* **utils:**
  *  Retry downloads until we get a success, or max retries ([23c84cd4](23c84cd4))
  *  Set timeout for waiting for a response to 1 day ([c5af2172](c5af2172))

#### Features

* **expire:**
  *  Display message if there's an error refreshing a tile ([73c66f7b](73c66f7b))
  *  Can change wait between checks of expire files ([319fa5b2](319fa5b2))
* **utils:**  Use proper rust errors for download & save ([7ea313f6](7ea313f6))



<a name="v0.4.0"></a>
## v0.4.0 (2016-05-25)


#### Bug Fixes

*   Correctly look at file age ([a7a9d51c](a7a9d51c))
*   Support current version of iter-progress ([b7236fbd](b7236fbd))
* **cache:**
  *  Set content-type header on responses ([5aed8bca](5aed8bca))
  *  Don't panic if you cannot download the tile, print error and continue ([7c71f53b](7c71f53b))
  *  Return file contents after saving ([14830392](14830392))

#### Features

*   Add --files-older-than to stuffer ([ab75f81d](ab75f81d))
* **cache:**
  *  Add cache control headers to response to limit caching ([e83bd472](e83bd472))
  *  Print if there's a cache miss/hit ([24e372d4](24e372d4))
  *  Print error if TileJSON is invalid ([524cf9f9](524cf9f9))
  *  Add --maxzoom option to test overzooming ([bc367ba9](bc367ba9))
* **serve:**  Add --urlprefix to override the URL in tilejson output ([ad54840a](ad54840a))



<a name="v0.3.0"></a>
## v0.3.0 (2016-03-22)


#### Features

*   Rename to iompair ([8b8ff07b](8b8ff07b))



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



