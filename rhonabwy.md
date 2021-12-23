### Rhonabwy

#### Build dependencies from source

##### [Jansson](https://github.com/akheron/jansson) (v2.14)

```
cd /PATH/TO/JANSSON/SRC_DIR
CC=/PATH/TO/gcc  CXX=/PATH/TO/g++  PKG_CONFIG_PATH="/PATH/TO/EXTRA/PKG_CONFIG_DIR" \
   cmake -DCMAKE_INSTALL_PREFIX=/PATH/TO/JANSSON/INSTALL_DIR  -DJANSSON_BUILD_SHARED_LIBS=on   ..
make
make install
```

##### [Nettle](https://github.com/gnutls/nettle) (v3.7.2)
```
cd /PATH/TO/NETTLE/SRC_DIR
./.bootstrap
./configure --prefix=/PATH/TO/NETTLE/INSTALL_DIR --disable-documentation  --enable-mini-gmp
make
make install
```
**NOTE**
* Documentation generation requires [texlive](https://github.com/TeX-Live) which consumes lots of disk space
* `--enable-mini-gmp` tells `nettle` to generate accompanied library **hogweed** (with flag `-lhogweed` at linking stage), which is required for later build process in [GnuTLS](.#gnutls)
* it is optional to compile code examples, you can skip it by modifying `all` target in `/PATH/TO/NETTLE/SRC_DIR/examples/Makefile.in`


##### [p11-kit](https://github.com/p11-glue/p11-kit) (v0.24.0)
This library relies on Python-based package [meson](https://github.com/mesonbuild/meson) for building other codebase
```
pip3 install meson ninja
```
Then
```
cd /PATH/TO/P11-KIT/SRC_DIR
mkdir _build
CC=gcc meson setup _build --prefix=/PATH/TO/P11-KIT/INSTALL_DIR
meson compile -C _build
meson test -C _build
meson install -C _build
```

**NOTE**
* it might not be necessary to use [`gettext`](https://packages.ubuntu.com/search?keywords=gettext) version 0.19 , you can downgrade `gettext` to version 0.18 by modifying `/PATH/TO/P11-KIT/SRC_DIR/configure.ac`

**to-do**
`p11-kit-trust.sh` test failed in [GnuTLS](.#gnutls) build process because it requires this package to generate `p11-kit-trust.so`, which is missing for unknown reason ?


##### [GnuTLS](#gnutls)
```
cd /PATH/TO/GNUTLS/SRC_DIR

./bootstrap

CC=gcc  PKG_CONFIG_PATH="/PATH/TO/P11-KIT/INSTALL_DIR/lib/x86_64-linux-gnu/pkgconfig:/PATH/TO/EXTRA/PKG_CONFIG_DIR" \
    ./configure  --prefix=/PATH/TO/GNUTLS/INSTALL_DIR  --with-included-libtasn1  --with-included-unistring  --disable-doc

LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/PATH/TO/NETTLE/INSTALL_DIR/lib:/PATH/TO/P11-KIT/INSTALL_DIR/lib/x86_64-linux-gnu"  make

LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/PATH/TO/NETTLE/INSTALL_DIR/lib:/PATH/TO/P11-KIT/INSTALL_DIR/lib/x86_64-linux-gnu"  make check

make install
```

**NOTE**
* `./bootstrap` will download source code of all dependcies , which may take few hours depending on your network speed
* it might not be necessary to use [`gettext`](https://packages.ubuntu.com/search?keywords=gettext) version 0.19 , you can downgrade `gettext` to version 0.18 by modifying `/PATH/TO/GNUTLS/SRC_DIR/configure.ac`



#### Build from source

```
cd /PATH/TO/RHONABWY/SRC_DIR

mkdir build

cd    build

CC=/PATH/TO/gcc  CXX=/PATH/TO/g++ PKG_CONFIG_PATH="/PATH/TO/JANSSON/INSTALL_DIR/lib/pkgconfig:/PATH/TO/GNUTLS/INSTALL_DIR/lib/pkgconfig:/PATH/TO/NETTLE/INSTALL_DIR/lib/pkgconfig:/PATH/TO/EXTRA/PKG_CONFIG_DIR"   cmake  -DWITH_JOURNALD=off  -DCMAKE_INSTALL_PREFIX=/PATH/TO/RHONABWY/INSTALL_DIR   ..

LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/PATH/TO/GNUTLS/INSTALL_DIR/lib: /PATH/TO/NETTLE/INSTALL_DIR/lib:/PATH/TO/P11-KIT/INSTALL_DIR/lib/x86_64-linux-gnu"  make

make install
```

