#### Build

```shell
cd /PATH/TO/PROJ/h2o
mkdir -p ./build ./installed
cd ./build

CC=/PATH/TO/bin/gcc  CXX=/PATH/TO/bin/g++  PKG_CONFIG_PATH="/PATH/TO/INSTALLED/libuv:/PATH/TO/INSTALLED/brotli" \
    cmake  -DCMAKE_INSTALL_PREFIX="/PATH/TO/INSTALLED/h2o"  -DBUILD_SHARED_LIBS=ON  -DCMAKE_BUILD_TYPE=Release \
    -DWITH_MRUBY=off ..

make

make libh2o
```

Note:
* `/PATH/TO/INSTALLED/libuv` and `/PATH/TO/INSTALLED/brotli` in `PKG_CONFIG_PATH` should contain [pkg-config](https://people.freedesktop.org/~dbn/pkg-config-guide.html) metadata file (usually the file with suffix name `.pc` , e.g. `libuv.pc`) for [libuv](https://github.com/libuv/libuv) and [brotli](https://github.com/google/brotli) accordingly.
* `BUILD_SHARED_LIBS = ON` indicates that cmake does NOT include some third-party libraries such as [brotli](https://github.com/google/brotli) when generating shared library `libh2o.so`, you'll need to generate shared library `libbrotlidec.so` and `libbrotlienc.so` for `brotli` by yourself.
* The optional parameter `CMAKE_BUILD_TYPE` can be omitted, which defaults to `Debug` and specifies `-O0` in C compile flags.


#### Run example
In `/PATH/TO/PROJ/h2o/build`, run the command :
```
make examples-simple-libuv
```

go back to parent folder (`/PATH/TO/PROJ/h2o`) , launch the sample server :
```
./build/examples-simple-libuv
```
test with web browser or command-line tools like curl

