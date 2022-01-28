
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

#### Streaming test
* streaming given video file to network protocol (in this example, UDP and h264 codec)
```
LD_LIBRARY_PATH="/PATH/TO/FFMPEG_INSTALLED_DIR/lib:/PATH/TO/GNUTLS_LIB_DIR:/PATH/TO/NETTLE_LIB_DIR:/PATH/TO/P11-KIT_LIB_DIR" \
    /PATH/TO/FFMPEG_INSTALLED_DIR/bin/ffmpeg -re  -i /PATH/TO/VIDEO_FILE -v 24  -c:v libx264 -c:a aac  -f mpegts   udp://127.0.0.1:12345
```
* `-v 24`  sets logging level to warning/error/fatal
* `-c:v` , `-c:a` depends on what video/audio codec supported in your ffmpeg build (if compile from source)
* `-f` may depend on format of input file (?)

* open another video player that supports streaming video
```
ffmpeg udp://127.0.0.1:12345
```
or
```
avplay udp://127.0.0.1:12345
```

#### Generate media segments for HLS protocol (HTTP Live Streaming)


### Reference
* [ffmpeg command usage & concept explanation](https://ffmpeg.org/ffmpeg.html)
* [API documentation for different versions](https://ffmpeg.org/documentation.html), Code examples are provided in the API documentation of each version
