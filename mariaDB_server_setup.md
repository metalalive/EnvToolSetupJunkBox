### MariaDB setup (build from source, and configure)

#### Envoronment
* Ubuntu 14.04LTS , Debian 9 Stretch (Raspbian Stretch)
* OpenSSL 1.1.1c, built from source
* MariaDB server 10.3.22

#### Pre-requisite
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

#### Download source
For those who work with limited disk space, It's suggested to do shallow clone from github
(no need to downlowd lots of useless old history commits)
```
cd  PATH/TO/MARIADB/SRC/FOLDER
git init
git remote add origin <URL/TO/MARIADB/GIT/REPO>
git fetch --depth 1 origin <COMMIT_SHA1>
git checkout FETCH_HEAD
```


#### Clean up before you build
* clean up the previous build (if exists). However it doesn't seem to reduce the size of `.git` ?
```
git clean -xffd
git submodule foreach --recursive git clean -xffd
```

#### Configure before building
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
    `WITHOUT_ROCKSDB=true`, `WITHOUT_TOKUDB=true`, `WITHOUT_MROONGA=true`, `WITHOUT_CONNECT=true`

```
cmake ..  -LH  -DBUILD_CONFIG=mysql_release   -DCMAKE_BUILD_TYPE=Debug \
    -DWITH_SSL=/PATH/TO/YOUR/OPENSSL/SRC/FOLDER  \
    -DCMAKE_INSTALL_PREFIX=/PATH/TO/INSTALLED/MARIADB/FOLDER  \
    -DWITH_UNIT_TESTS=OFF \
    >& cmake.log &
```

#### Modify source files if building with GCC >= 6.3

Build errors will happen on the OS with GCC toolchain verison >= 6.3 , for example Raspbian Stretch. 
MariaDB sets build option `-Werror` to compile each file, all warnings will be treated as error.

##### Uninitialized warnings
```
MARIADB_SRC_PATH/mysys/my_context.c: In function ‘my_context_spawn’:
MARIADB_SRC_PATH/mysys/my_context.c:106:3: error: ‘u.a[1]’ may be used uninitialized in this function [-Werror=maybe-uninitialized]
   makecontext(&c->spawned_context, my_context_spawn_internal, 2,
   ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
               u.a[0], u.a[1]);
               ~~~~~~~~~~~~~~~
cc1: all warnings being treated as errors
```
* By giving initial value zero to unused `u.a[1]`, the warning is gone.
* Modify both files `mysys/my_context.c`, and `libmariadb/libmariadb/ma_context.c`
 
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

##### Redefined parameters between OpenSSL and MariaDB
```
/PATH/TO/OPENSSL/SRC/include/openssl/crypto.h:200:0: error: "CRYPTO_cleanup_all_ex_data" redefined [-Werror]
 # define CRYPTO_cleanup_all_ex_data() while(0) continue 
In file included from MARIADB_SRC_PATH/mysys_ssl/openssl.c:18:0:
MARIADB_SRC_PATH/include/ssl_compat.h:42:0: note: this is the location of the previous definition
 #define CRYPTO_cleanup_all_ex_data()
 
In file included from MARIADB_SRC_PATH/mysys_ssl/openssl.c:33:0:
/PATH/TO/OPENSSL/SRC/include/openssl/evp.h:538:0: error: "EVP_MD_CTX_init" redefined [-Werror]
 # define EVP_MD_CTX_init(ctx)    EVP_MD_CTX_reset((ctx))
In file included from MARIADB_SRC_PATH/mysys_ssl/openssl.c:18:0:
MARIADB_SRC_PATH/include/ssl_compat.h:29:0: note: this is the location of the previous definition
 #define EVP_MD_CTX_init(X) do { memset((X), 0, EVP_MD_CTX_SIZE); EVP_MD_CTX_reset(X); } while(0)

In file included from MARIADB_SRC_PATH/mysys_ssl/openssl.c:33:0:
/PATH/TO/OPENSSL/SRC/include/openssl/evp.h:672:0: error: "EVP_CIPHER_CTX_init" redefined [-Werror]
 #  define EVP_CIPHER_CTX_init(c)      EVP_CIPHER_CTX_reset(c) 
In file included from MARIADB_SRC_PATH/mysys_ssl/openssl.c:18:0:
MARIADB_SRC_PATH/include/ssl_compat.h:31:0: note: this is the location of the previous definition
 #define EVP_CIPHER_CTX_init(X) do { memset((X), 0, EVP_CIPHER_CTX_SIZE); EVP_CIPHER_CTX_reset(X); } while(0)

In file included from MARIADB_SRC_PATH/mysys_ssl/openssl.c:33:0:
/PATH/TO/OPENSSL/SRC/include/openssl/evp.h:958:0: error: "EVP_cleanup" redefined [-Werror]
 #  define EVP_cleanup() while(0) continue
In file included from MARIADB_SRC_PATH/mysys_ssl/openssl.c:18:0:
MARIADB_SRC_PATH/include/ssl_compat.h:40:0: note: this is the location of the previous definition
 #define EVP_cleanup()
```

* Workaround : comment off the refined parameters `CRYPTO_cleanup_all_ex_data`, `EVP_MD_CTX_init`,
  `EVP_CIPHER_CTX_init`, `EVP_cleanup`, on openssl side. 
* [TODO] Better solution would be to recheck C header include sequence.


##### Shift count error
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

* Workaround : explicitly declare 64-bit varible for `def_temp2` in `ma_global.h` , for example :

```
#define int8store(T,A)       do { uint def_temp= (uint) (A); \
                                 unsigned long long def_temp2= ((unsigned long long) (A)) >> 32; \
                             } while(0);
```
**NOTE**:
* modify both files `MARIADB_SRC_PATH/libmariadb/include/ma_global.h`, and `MARIADB_SRC_PATH/include/byte_order_generic.h`
* `unsigned long long` may be defined in somewhere of this repo ?


##### Cast alignment error in mroonga (groonga)
```
[ 66%] Building C object storage/mroonga/vendor/groonga/lib/CMakeFiles/libgroonga.dir/alloc.c.o
MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/lib/alloc.c: In function ‘grn_ctx_alloc’:
MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/lib/alloc.c:419:16: error: cast increases required alignment of target type [-Werror=cast-align]
       header = (int32_t *)((byte *)mi->map + mi->nref);
                ^
cc1: all warnings being treated as errors
```
or
```
[ 74%] Building C object storage/mroonga/vendor/groonga/plugins/suggest/CMakeFiles/suggest.dir/suggest.c.o
In file included from MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/include/groonga.h:22:0,
                 from MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/lib/grn.h:759,
                 from MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/lib/grn_ctx.h:21,
                 from MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/plugins/suggest/suggest.c:24:
MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/plugins/suggest/suggest.c: In function ‘cooccurrence_search’:
MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/include/groonga/groonga.h:1475:34: error: cast increases required alignment of target type [-Werror=cast-align]
 #define GRN_RECORD_VALUE(obj) (*((grn_id *)GRN_BULK_HEAD(obj)))
```

* It seems that there's no need to add `-Wcast-align` in `MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/CMakeLists.txt`
* It's commented off in [Groonga repo](https://github.com/groonga/groonga/blob/master/CMakeLists.txt), [(see this commit)](https://github.com/groonga/groonga/commit/65fd6d0b599ee1b120caa2ecc3bd9e17eae4695e#diff-af3b638bc2a3e6c650974192a53c7291) . However in [MariaDB server repo](https://github.com/MariaDB/server/blob/2f7d91bb6ce7bb34dd644e30590189bce37fb8f1/storage/mroonga/vendor/groonga/CMakeLists.txt#L161) , `-Wcast-align` is still set.
* Workaround : simply remove `-Wcast-align`  in :
  * `MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/CMakeLists.txt`
  * `MARIADB_SRC_PATH/storage/mroonga/vendor/groonga/configure.ac`


##### Build

* Simply run `make`
* Build process takes about 24 hours in Raspberry PI 1, for Intel core 5, it takes about an hour.
* Minimum disk space required : 5GB


##### Install

* Simply run `make install`
* It's OK to run `make install` without `root` privilege,  then you must ensure that current user account  of your target Linux system has full access permission to the path `CMAKE_INSTALL_PREFIX`.
* If you install mariadb on mounted disk (e.g. external USB disk), you must ensure the mounted disk is **NOT** Windows filesystem e.g. `FAT`/`FAT32`/`NTFS` [(reference)](https://askubuntu.com/questions/1111542/cant-change-ownership-of-mounted-device), otherwise you'd get permission denied error during installation.  
* Minimum disk space required : 2.3GB

#### Configuration after Installation

To start `mysqld` at boot time you have to copy `support-files/mysql.server` to the right place for your system

PLEASE REMEMBER TO SET A PASSWORD FOR THE MariaDB root USER ! To do so, start the server, then issue the following commands:
```
./bin/mysqladmin -u root password 'new-password'
./bin/mysqladmin -u root -h localhost password 'new-password'
```

Alternatively you can run `./bin/mysql_secure_installation`, which will also give you the option of removing the test
databases and anonymous user created by default.  This is strongly recommended for production servers.

See the MariaDB Knowledgebase at http://mariadb.com/kb or the MySQL manual for more instructions.

You can start the MariaDB daemon with:
```
./bin/mysqld_safe --datadir='./data'
```

You can test the MariaDB daemon with `mysql-test-run.pl`
```
cd './mysql-test' ; perl mysql-test-run.pl
```

Please report any problems at http://mariadb.org/jira


-----------------------------------------------------------------------

### Useful commands for database management

##### start MariaDB server:
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  /PATH/TO/MARIADB/INSTALL/bin/mysqld_safe  --datadir='PATH/TO/DATABASE/FOLDER'  &
```

##### Shutdown MariaDB server
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  kill -SIGTERM <MARIADB_PID>
```

##### Login MariaDB through interactive command line interface :
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  /PATH/TO/MARIADB/INSTALL/bin/mysql -u  <USER_ACCOUNT> -p -h <IP_OR_DOMAIN_NAME>
```

##### List all existing databases
```
SHOW DATABASES;
```

##### switch to specific database
```
USE <YOUR_DATABASE_NAME>;
```

##### List all columns (and their attributes) of a database table
```
SHOW COLUMNS FROM <YOUR_TABLE_NAME>;
```


##### List attributes of all available database users
```
SELECT  host,user,max_connections,max_user_connections  FROM mysql.user
```

##### Check user privilege
Note don't list privilege fields of any user in `mysql.user`, they're NOT synchronized with `GRANT` command
```
SHOW GRANTS FOR  YOUR_USER_ACCOUNT@YOUR_HOSTNAME;
```

##### Grant privilege
Grant certain type(s) of privilege to specific database for specific user.
```
GRANT CREATE,DROP,INDEX, ANY_VALID_PRIVILEGE_OPTIONS  ON \
     `DATABASE_NAME`.* TO 'DB_USERNAME'@'IP_OR_DOMAIN_NAME';
```

It can also grant privileges to specific database table by modifying :
```
`DATABASE_NAME`.*
```
to 
```
`DATABASE_NAME`.`TABLE_NAME`
```

##### List table size of a specific database in descending order
```
SELECT table_name, ROUND(((data_length + index_length) / 1024), 2) `Size (KB)`\
    FROM information_schema.TABLES \
    WHERE TABLE_SCHEMA = 'YOUR_DATABASE_NAME' \
    ORDER BY (data_length + index_length)  DESC;
```

#### Drop all tables in a database, but still keep database itself
```
SELECT CONCAT('DROP TABLE IF EXISTS `', TABLE_SCHEMA, '`.`', TABLE_NAME, '`;') \
FROM information_schema.TABLES WHERE TABLE_SCHEMA = 'mydb';
```


#### Reference
* [Get the code, build it, test it](https://mariadb.org/get-involved/getting-started-for-developers/get-code-build-test/)
* [How To Reset Your MySQL or MariaDB Root Password](https://www.digitalocean.com/community/tutorials/how-to-reset-your-mysql-or-mariadb-root-password)
