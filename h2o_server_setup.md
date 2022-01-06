

```shell
cd /CUSTOM/PATH/TO/h2o
mkdir -p ./build ./installed
cd ./build

CC=/CUSTOM/PATH/TO/bin/gcc  CXX=/CUSTOM/PATH/TO/bin/g++  PKG_CONFIG_PATH=/CUSTOM/PATH/TO/libuv  cmake  -DCMAKE_INSTALL_PREFIX="/CUSTOM/PATH/TO/h2o/installed"  -DBUILD_SHARED_LIBS="ON"  -DWITH_MRUBY=off ..

make

make libh2o

make examples-simple-libuv
```

Note:
* `/CUSTOM/PATH/TO/libuv` should contain metadata file (usually the file with suffix name `.pc` , e.g. `libuv.pc`) for [pkg-config](https://people.freedesktop.org/~dbn/pkg-config-guide.html)
* `BUILD_SHARED_LIBS` indicates that cmake does NOT include some third-party libraries such as [brotli](https://github.com/google/brotli) when generating shared library `libh2o.so`, you'll need to generate shared library for `brotli` by yourself.

