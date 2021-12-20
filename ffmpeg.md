
#### Build from source
```
cd /PATH/TO/FFMPEG_REPO_HOME

mkdir -p installed

./configure --prefix=/PATH/TO/FFMPEG_REPO_HOME/installed --enable-libx264 --enable-libx265 \
        --enable-libfdk-aac  --enable-gnutls --enable-gpl --enable-nonfree --enable-shared --disable-static

make

make install
```
* You can add mode codec libraires supported in the valid configuration options (in `/PATH/TO/FFMPEG_REPO_HOME/configure`)
* To build static libraries, omit `--disable-static`
* `--enable-gnutls`, `--enable-gpl` are essential for H.264 codec libraries (`libx264-dev` in Ubuntu)
* compilation takes about 30 minutes depending on your environment
* After `make install` , all essential files (e.g. headers, libraries, pkg-config files ... etc) are in `/PATH/TO/FFMPEG_REPO_HOME/installed`

To test `ffmpeg` command :
```
LD_LIBRARY_PATH="/PATH/TO/FFMPEG_REPO_HOME/installed/lib"  /PATH/TO/FFMPEG_REPO_HOME/installed/bin/ffmpeg --help
```


#### Reference
* [ffmpeg command usage & concept explanation](https://ffmpeg.org/ffmpeg.html)
* [API documentation for different versions](https://ffmpeg.org/documentation.html), Code examples are provided in the API documentation of each version
