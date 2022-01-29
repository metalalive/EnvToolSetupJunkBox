
### Build from source
```shell
cd /PATH/TO/FFMPEG_REPO_HOME

mkdir -p installed

LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/PATH/TO/NETTLE_LIB_DIR:/PATH/TO/P11-KIT_LIB_DIR" \
    PKG_CONFIG_PATH="/PATH/TO/GNUTLS_PKG_CONFIG_DIR:/PATH/TO/NETTLE_PKG_CONFIG_DIR:/usr/lib/x86_64-linux-gnu/pkgconfig" \
    ./configure --prefix=/PATH/TO/FFMPEG_INSTALLED_DIR --enable-libx264 --enable-libx265 \
    --enable-libfdk-aac  --enable-gnutls --enable-gpl --enable-nonfree --enable-shared --disable-static

LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/PATH/TO/GNUTLS_LIB_DIR:/PATH/TO/NETTLE_LIB_DIR:/PATH/TO/P11-KIT_LIB_DIR"   make

make install
```
* You can add mode codec libraires supported in the valid configuration options (in `/PATH/TO/FFMPEG_REPO_HOME/configure`)
* To build static libraries, omit `--disable-static`
* `--enable-gnutls`, `--enable-gpl` are essential for H.264 codec libraries (`libx264-dev` in Ubuntu)
* compilation takes about 30 minutes depending on your environment
* After `make install` , all essential files (e.g. headers, libraries, pkg-config files ... etc) are in `/PATH/TO/FFMPEG_REPO_HOME/installed`

### Testing
#### Help document
```
LD_LIBRARY_PATH="/PATH/TO/FFMPEG_INSTALLED_DIR/lib:/PATH/TO/GNUTLS_LIB_DIR:/PATH/TO/NETTLE_LIB_DIR:/PATH/TO/P11-KIT_LIB_DIR" \
    /PATH/TO/FFMPEG_INSTALLED_DIR/bin/ffmpeg --help <OPTIONAL_MORE_DETAIL>
```
*  `<OPTIONAL_MORE_DETAIL>` indicates that more detail can be dumped to terminal, which could be `--help long` or `--help full` (dump all detail used in codec settings)

#### Live Streaming test
* streaming given video file (or address to webcam) to network protocol (in this example, UDP and h264 codec)
```
LD_LIBRARY_PATH="/PATH/TO/FFMPEG_INSTALLED_DIR/lib:/PATH/TO/GNUTLS_LIB_DIR:/PATH/TO/NETTLE_LIB_DIR:/PATH/TO/P11-KIT_LIB_DIR" \
    /PATH/TO/FFMPEG_INSTALLED_DIR/bin/ffmpeg -re  -i /PATH/TO/VIDEO_FILE -v 24  -c:v libx264 -c:a aac  -f mpegts   udp://127.0.0.1:12345
```
* `-v 24`  sets logging level to warning/error/fatal
* `-c:v` , `-c:a` depends on what video/audio codec supported in your ffmpeg build (if compile from source) . Check all supported codecs in your environment by the command `/PATH/TO/FFMPEG_INSTALLED_DIR/bin/ffmpeg -codecs`.
* `-f` may depend on format of input file (?)
* the streaming process  terminates as soon as it reaches end of frames.

* open another video player that supports streaming video
```
ffmpeg udp://127.0.0.1:12345
```
or
```
avplay udp://127.0.0.1:12345
```
* the player won't start playing from the beginning once it is reopened.

#### HTTP Live Streaming
Given video file in local disk, generate playlist and corresponding media segments which are compliant with [HLS protocol](https://datatracker.ietf.org/doc/html/rfc8216) 
```
LD_LIBRARY_PATH="/PATH/TO/FFMPEG_INSTALLED_DIR/lib:/PATH/TO/GNUTLS_LIB_DIR:/PATH/TO/NETTLE_LIB_DIR:/PATH/TO/P11-KIT_LIB_DIR" \
    /PATH/TO/FFMPEG_INSTALLED_DIR/bin/ffmpeg   -i /PATH/TO/LOCAL/VIDEO.mp4 -v 24 -c:v libx264 -preset slow  -c:a aac \
    -b:a 128k -ac 2 -f hls -hls_time 11 -hls_playlist_type vod -hls_segment_type fmp4   stream.m3u8
```
* `-f hls` indicates the output should be HLS compliant
* `-hls_segment_type` can be either  [`fmp4`](https://datatracker.ietf.org/doc/html/rfc8216#section-3.3) or [`ts`](https://datatracker.ietf.org/doc/html/rfc8216#section-3.2) 
* The output includes a playlist file named ` stream.m3u8` , a set of media segment files with default name `stream0.m4s`, `stream1.m4s`, `stream2.m4s` ..... etc (for `-hls_segment_type fmp4`)
* `-hls_playlist_type` can be `vod` or `event`, `vod` is for static video file, `event` is for live stream because applicatoins is allowed to append new media segment entries to the end of playlist (the `.m3u8` file)
* the input file can also have multiple video streams (possible usage scenario could be to provide different resolution and bitrate for different network speed)
* File name and URL pattern of each media segment can be changed via `-hls_segment_filename` and `-hls_base_url` option respectively.

After the command, you can host all the output files on HTTP server and test them with video player in a HTML file (see the sample below, also hosted on the same server) . Note that [`hls.js`](https://github.com/video-dev/hls.js/) in the HTML sample can be copied from [the CDN server](https://cdn.jsdelivr.net/npm/hls.js@latest)
```html
<!DOCTYPE html>
<html>
<body>
<script src="./hls.js"></script>
<video id="video_test_123" width="640" height="320" controls>
</video>
<script>
  if(Hls.isSupported()) {
    var video = document.getElementById('video_test_123');
    var hls = new Hls();
    hls.loadSource('./stream.m3u8');
    hls.attachMedia(video);
    hls.on(Hls.Events.MANIFEST_PARSED,function() {
      video.play();
  });
 }
</script>
</body>
</html>
```


### Reference
* [ffmpeg command usage & concept explanation](https://ffmpeg.org/ffmpeg.html)
* [API documentation for different versions](https://ffmpeg.org/documentation.html), Code examples are provided in the API documentation of each version
