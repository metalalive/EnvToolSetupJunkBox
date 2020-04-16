### MariaDB setup (build from source, and configure)

##### Envoronment
* Ubuntu 14.04LTS , Debian 9 Stretch (Raspbian Stretch)
* OpenSSL 1.1.1c, built from source
* MariaDB server 10.3.22

##### Pre-requisite
* To avoid `jemalloc` not found when running `cmake` on 64-bit CPU platform.
```
apt-get install libjemalloc-dev
```

* To avoid `AUTH_PAM` build failure when running `cmake`
```
apt-get  install libpam0g-dev
```

* To avoid `LibXml2` not found when running `cmake`
```
apt-get install libxml2-dev
```


##### Download source
For those who work with limited disk space, It's suggested to do shallow clone from github
(no need to downlowd lots of useless old history commits)
```
cd  PATH/TO/MARIADB/SRC/FOLDER
git init
git remote add origin <URL/TO/MARIADB/GIT/REPO>
git fetch --depth 1 origin <COMMIT_SHA1>
git checkout FETCH_HEAD
```


##### Clean up before you build
* clean up the previous build (if exists). However it doesn't seem to reduce the size of `.git` ?
```
git clean -xffd
git submodule foreach --recursive git clean -xffd
```

##### Configure before building
* In CMake you can turn off AUTH_PAM, it may not be essential in this version (default is "ON"), it can be
  disabled by setting `SET(PLUGIN_AUTH_PAM NO)`  at :
  `<MARIADB_SRC_FOLDER>/cmake/build_configurations/mysql_release.cmake`

* then run `cmake`, note that :
  * `CMAKE_BUILD_TYPE` can be `Release`, `Debug`, `RelWithDebInfo`, 
  * It would be better to create standalone folder for `CMAKE_INSTALL_PREFIX` instead of installing all
    the built files directly  to `/usr/local` or `/usr`, for easy uninstallztion in the future (by simply
    remove the standslone folder which stores all the built files)
  * `WITH_UNIT_TESTS` can be `on` (default) and `off`, turn `off` if you don't need it
  * optional storage engines can be `off`, e.g:
    `WITHOUT_ROCKSDB=true`, `WITHOUT_TOKUDB=true`, `WITHOUT_MROONGA=true`
```
cmake ..  -LH  -DBUILD_CONFIG=mysql_release   -DCMAKE_BUILD_TYPE=Debug \
    -DWITH_SSL=/PATH/TO/YOUR/OPENSSL/SRC/FOLDER  \
    -DCMAKE_INSTALL_PREFIX=/PATH/TO/INSTALLED/MARIADB/FOLDER  \
    -DWITH_UNIT_TESTS=OFF \
    >& cmake.log &
```

##### Modify source files if building with GCC >= 6.3

Build errors will happen on the OS with GCC toolchain verison >= 6.3 , for example Raspbian Stretch. 
MariaDB sets build option `-Werror` to compile each file, all warnings will be treated as error.

###### Uninitialized warnings
  at `/PATH/TO/MARIADB/SRC/mysys/my_context.c`
```
[ 10%] Building C object mysys/CMakeFiles/mysys.dir/my_rdtsc.c.o
[ 10%] Building C object mysys/CMakeFiles/mysys.dir/my_context.c.o
/PATH/TO/MARIADB/SRC/mysys/my_context.c: In function ‘my_context_spawn’:
/PATH/TO/MARIADB/SRC/mysys/my_context.c:106:3: error: ‘u.a[1]’ may be used uninitialized in this function [-Werror=maybe-uninitialized]
   makecontext(&c->spawned_context, my_context_spawn_internal, 2,
   ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
               u.a[0], u.a[1]);
               ~~~~~~~~~~~~~~~
cc1: all warnings being treated as errors
mysys/CMakeFiles/mysys.dir/build.make:2606: recipe for target 'mysys/CMakeFiles/mysys.dir/my_context.c.o' failed
```
 By giving initial value zero to unused `u.a[1]`, the warning is gone.
 ```
 my_context_spawn(struct my_context *c, void (*f)(void *), void *d)
 {
     int err;
     union pass_void_ptr_as_2_int u;
     u.a[1] = 0;
     ..............
     makecontext(&c->spawned_context, my_context_spawn_internal, 2,
         u.a[0], u.a[1]);
     ...............
}

```

###### Redefined parameters between OpenSSL and MariaDB
```
/PATH/TO/OPENSSL/SRC/include/openssl/crypto.h:200:0: error: "CRYPTO_cleanup_all_ex_data" redefined [-Werror]
 # define CRYPTO_cleanup_all_ex_data() while(0) continue 
In file included from /PATH/TO/MARIADB/SRC/mysys_ssl/openssl.c:18:0:
/PATH/TO/MARIADB/SRC/include/ssl_compat.h:42:0: note: this is the location of the previous definition
 #define CRYPTO_cleanup_all_ex_data()
 
In file included from /PATH/TO/MARIADB/SRC/mysys_ssl/openssl.c:33:0:
/PATH/TO/OPENSSL/SRC/include/openssl/evp.h:538:0: error: "EVP_MD_CTX_init" redefined [-Werror]
 # define EVP_MD_CTX_init(ctx)    EVP_MD_CTX_reset((ctx))
In file included from /PATH/TO/MARIADB/SRC/mysys_ssl/openssl.c:18:0:
/PATH/TO/MARIADB/SRC/include/ssl_compat.h:29:0: note: this is the location of the previous definition
 #define EVP_MD_CTX_init(X) do { memset((X), 0, EVP_MD_CTX_SIZE); EVP_MD_CTX_reset(X); } while(0)

In file included from /PATH/TO/MARIADB/SRC/mysys_ssl/openssl.c:33:0:
/PATH/TO/OPENSSL/SRC/include/openssl/evp.h:672:0: error: "EVP_CIPHER_CTX_init" redefined [-Werror]
 #  define EVP_CIPHER_CTX_init(c)      EVP_CIPHER_CTX_reset(c) 
In file included from /PATH/TO/MARIADB/SRC/mysys_ssl/openssl.c:18:0:
/PATH/TO/MARIADB/SRC/include/ssl_compat.h:31:0: note: this is the location of the previous definition
 #define EVP_CIPHER_CTX_init(X) do { memset((X), 0, EVP_CIPHER_CTX_SIZE); EVP_CIPHER_CTX_reset(X); } while(0)

In file included from /PATH/TO/MARIADB/SRC/mysys_ssl/openssl.c:33:0:
/PATH/TO/OPENSSL/SRC/include/openssl/evp.h:958:0: error: "EVP_cleanup" redefined [-Werror]
 #  define EVP_cleanup() while(0) continue
In file included from /PATH/TO/MARIADB/SRC/mysys_ssl/openssl.c:18:0:
/PATH/TO/MARIADB/SRC/include/ssl_compat.h:40:0: note: this is the location of the previous definition
 #define EVP_cleanup()
```

workaround : comment off the refined parameters `CRYPTO_cleanup_all_ex_data`, `EVP_MD_CTX_init`, `EVP_CIPHER_CTX_init`, `EVP_cleanup`, on openssl side. 

**[TODO] Better solution would be to recheck C header include sequence.**


###### Shift count error

```
[ 42%] Building C object libmariadb/libmariadb/CMakeFiles/mariadb_obj.dir/mariadb_stmt.c.o
In file included from MARIADB_SRC_PATH/libmariadb/libmariadb/mariadb_stmt.c:46:0:
MARIADB_SRC_PATH/libmariadb/libmariadb/mariadb_stmt.c: In function ‘mysql_net_store_length’:
MARIADB_SRC_PATH/libmariadb/include/ma_global.h:914:85: error: right shift count >= width of type [-Werror=shift-count-overflow]
 #define int8store(T,A)       do { uint def_temp= (uint) (A), def_temp2= (uint) ((A) >> 32); \
                                                                                     ^
MARIADB_SRC_PATH/libmariadb/libmariadb/mariadb_stmt.c:476:3: note: in expansion of macro ‘int8store’
   int8store(packet, length);
   ^~~~~~~~~
cc1: all warnings being treated as errors
```

workaround : explicitly declare 64-bit varible for `def_temp2` in `ma_global.h` , for example :

```
#define int8store(T,A)       do { uint def_temp= (uint) (A); \
                                 unsigned long long def_temp2= ((unsigned long long) (A)) >> 32; \
                             } while(0);
```



##### Build
```
make
```


######
remove generated build option "-Wcast-align" in following make files :
./storage/mroonga/vendor/groonga/lib/CMakeFiles/libgroonga.dir/flags.make
./storage/mroonga/vendor/groonga/plugins/suggest/CMakeFiles/suggest.dir/flags.make
./storage/mroonga/vendor/groonga/plugins/functions/CMakeFiles/index_column_functions.dir/flags.make
./storage/mroonga/vendor/groonga/plugins/functions/CMakeFiles/math_functions.dir/flags.make
./storage/mroonga/vendor/groonga/plugins/functions/CMakeFiles/number_functions.dir/flags.make
./storage/mroonga/vendor/groonga/plugins/functions/CMakeFiles/string_functions.dir/flags.make
./storage/mroonga/vendor/groonga/plugins/functions/CMakeFiles/time_functions.dir/flags.make
./storage/mroonga/vendor/groonga/plugins/functions/CMakeFiles/vector_functions.dir/flags.make



as the commit in Groonga repo:
https://github.com/groonga/groonga/commit/65fd6d0b599ee1b120caa2ecc3bd9e17eae4695e#diff-af3b638bc2a3e6c650974192a53c7291

seems that -Wcast-align in mroonga doesn't match groonga
https://github.com/MariaDB/server/commit/13167e64898da6373fa8cab2ad89514eaf886412#diff-9f0f041990da73f0cbc97a7aafdfadccL163-L164


------ MariaDB (Ubuntu 14.04) ---------------

cmake ..  -LH  -DBUILD_CONFIG=mysql_release   -DCMAKE_BUILD_TYPE=Debug  -DWITH_SSL=/opt/custom/projects/c/security/openssl/  -DCMAKE_INSTALL_PREFIX=/usr/local/mariadb   >& cmake.log &




https://mariadb.org/get-involved/getting-started-for-developers/get-code-build-test/
