#### Build

```shell
cd /PATH/TO/PROJ/h2o
mkdir -p ./build ./installed
cd ./build

CC=/PATH/TO/bin/gcc  CXX=/PATH/TO/bin/g++  PKG_CONFIG_PATH="/PATH/TO/INSTALLED/libuv:/PATH/TO/INSTALLED/brotli" \
    cmake  -DCMAKE_INSTALL_PREFIX="/PATH/TO/INSTALLED/h2o"  -DBUILD_SHARED_LIBS=ON  -DCMAKE_BUILD_TYPE=Release \
    -DWITH_MRUBY=off  -DWITH_FUSION=OFF  ..
```
To generate only the library, you have:
```shell
make libh2o
```
To generate only the executable standalone server, you have:
```shell
make h2o
```

Note:
- `/PATH/TO/INSTALLED/libuv` and `/PATH/TO/INSTALLED/brotli` in `PKG_CONFIG_PATH` should contain [pkg-config](https://people.freedesktop.org/~dbn/pkg-config-guide.html) metadata file (usually the file with suffix name `.pc` , e.g. `libuv.pc`) for [libuv](https://github.com/libuv/libuv) and [brotli](https://github.com/google/brotli) accordingly.
- `BUILD_SHARED_LIBS = ON` indicates that cmake does NOT include some third-party libraries such as [brotli](https://github.com/google/brotli) when generating shared library `libh2o.so`, you'll need to generate shared library `libbrotlidec.so` and `libbrotlienc.so` for `brotli` by yourself.
- The optional parameter `CMAKE_BUILD_TYPE` can be omitted, which defaults to `Debug` and specifies `-O0` in C compile flags.
- The cmake option `WITH_FUSION` indicates a feature that accelerates [AEAD](https://en.wikipedia.org/wiki/Authenticated_encryption) operation in secure connection, this feature will be included ONLY in standalone server (not in library `libh2o`, it comes from [picotls](https://github.com/h2o/picotls)) . Also this feature requires hardware support (e.g. dedicated [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard) instruction) and currently works well only on some of x86 CPUs.


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

#### Run test
prerequisites:
- perl environment, install essential modules
- internet connectivity, few of test cases perform DNS resolution
- MRuby may need to be enabled, few of test cases seem to require it

run unit test and integration test (total running time ~ 30 minutes)
```
make check
```


#### Generate document
prerequisites:
- perl environment, install the modules (using `cpan` CLI) e.g. `Test::More`, `HTML::TokeParser`, `Text::MicroTemplate`
- oktivia, just clone the git repo from [here](https://github.com/shibukawa/oktavia/tree/25b615f5bc0902b9107e3ffc1178ec6bf768b0ad) to `/misc/oktivia` (by default it is empty), for pre-built executable

modify the page in `/srcdoc/*.t`, regenerate the html using the command below once you finish editing.

```
make doc
```
