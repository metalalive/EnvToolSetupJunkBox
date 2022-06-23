#### Build

create directory ofr a new build of the example code
```
cd  /PATH/TO/EXAMPLE
mkdir ./build
cd  ./build
```

the example uses `CMake` and `pkg-config` to build executable
```
CC="/PATH/TO/gcc/10.3.0/installed/bin/gcc"  PKG_CONFIG_PATH="[1]:[2]:[3]:[4]"  cmake ..
make  custom_avio_transcoding_mp4
```
Note that `PKG_CONFIG_PATH` has to include 3rd-party libraries applied to this example code. These are :

[1] GnuTLS, 3.7.2 `/PATH/TO/gnutls/3.7.2/installed/lib/pkgconfig`
[2] Nettle, dependency library used in GnuTLS `/PATH/TO/gnutls/3.7.2/devel/nettle/installed/lib/pkgconfig`
[3] p11-kit, 0.24.0 `/PATH/TO/p11-kit/0.24.0/installed/lib/x86_64-linux-gnu/pkgconfig`
[4] ffmpeg, 4.3.3 `/PATH/TO/ffmpeg/4.3.3/installed/lib/pkgconfig`


#### Run
```
LD_LIBRARY_PATH="$LD_LIBRARY_PATH:[1]:[2]:[3]:[4]"   ./custom_avio_transcoding_mp4 \
     <INPUT_FILE_PATH>   <OUTPUT_FILE_PATH>   <NUM_PACKETS_TRANSCODED>
```
[1] GnuTLS, 3.7.2 `/PATH/TO/gnutls/3.7.2/installed/lib/`
[2] Nettle, dependency library used in GnuTLS `/PATH/TO/gnutls/3.7.2/devel/nettle/installed/lib/`
[3] p11-kit, 0.24.0 `/PATH/TO/p11-kit/0.24.0/installed/lib/x86_64-linux-gnu/`
[4] ffmpeg, 4.3.3 `/PATH/TO/ffmpeg/4.3.3/installed/lib/`

