
#### Build from source
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

To test `ffmpeg` command :
```
LD_LIBRARY_PATH="/PATH/TO/FFMPEG_INSTALLED_DIR/lib:/PATH/TO/GNUTLS_LIB_DIR:/PATH/TO/NETTLE_LIB_DIR:/PATH/TO/P11-KIT_LIB_DIR" \
    /PATH/TO/FFMPEG_INSTALLED_DIR/bin/ffmpeg --help
```


#### Reference
* [ffmpeg command usage & concept explanation](https://ffmpeg.org/ffmpeg.html)
* [API documentation for different versions](https://ffmpeg.org/documentation.html), Code examples are provided in the API documentation of each version
