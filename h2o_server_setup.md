

```shell
CC=/CUSTOM/PATH/TO/bin/gcc  CXX=/CUSTOM/PATH/TO/bin/g++  PKG_CONFIG_PATH=/CUSTOM/PATH/TO/libuv  cmake .. -DWITH_MRUBY=off

make

make libh2o
```

Note that `/CUSTOM/PATH/TO/libuv` should contain metadata file (usually the file with suffix name `.pc` , e.g. `libuv.pc`) for [pkg-config](https://people.freedesktop.org/~dbn/pkg-config-guide.html)


