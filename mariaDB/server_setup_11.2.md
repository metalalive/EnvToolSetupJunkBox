## MariaDB build from source and configuration
MariaDB version 11.2.3

### Envoronment
* Ubuntu LTS
* GCC 10.3.0
* OpenSSL 3.1.4, built from source


### Pre-requisite
Check the libraies required by the build [in this doc](./mariaDB/server_setup_10.3.md#pre-requisite) 

### Download source
For those who work with limited disk space, It's suggested to do shallow clone from github.
Check out the instructions at [here](./git_setup.md).

Note some of subfolders are linked to other git repositories,
please ensure these dependent git codebases are also cloned to your local environment.

### Clean up before build
* clean up the previous build (if exists). However it doesn't seem to reduce the size of `.git` ?
```
git clean -xffd
git submodule foreach --recursive git clean -xffd
```

### Configuration
Run `cmake`

```bash
CC="/PATH/TO//gcc/10.3.0/bin/gcc"  cmake .. -LH \
    -DBUILD_CONFIG=mysql_release   -DCMAKE_BUILD_TYPE=Debug \
    -DWITH_SSL=/PATH/TO/YOUR/OPENSSL/SRC/FOLDER  \
    -DCMAKE_INSTALL_PREFIX=/PATH/TO/INSTALLED/MARIADB/FOLDER  \
    -DWITH_UNIT_TESTS=OFF \
    >& cmake.log &
```

Note that :
- `CMAKE_BUILD_TYPE` can be `Release`, `Debug`, `RelWithDebInfo`, 
- It would be better to create standalone folder for `CMAKE_INSTALL_PREFIX` instead of installing all
    the built files directly  to `/usr/local` or `/usr`, for easy uninstallztion in the future (by simply
    remove the standslone folder which stores all the built files)
- `WITH_UNIT_TESTS` can be `on` (default) and `off`, turn `off` if you don't need it
- optional storage engines can be `off`, e.g:
  `WITHOUT_ROCKSDB=true`, `WITHOUT_TOKUDB=true`, `WITHOUT_MROONGA=true`, `WITHOUT_CONNECT=true`

#### Modify source files if building with GCC >= 6.3

Build errors will happen on the OS with GCC toolchain verison >= 6.3 , for example Raspbian Stretch. 
MariaDB sets build option `-Werror` to compile each file, all warnings will be treated as error.

##### Build
- Simply run `make`
- Build process takes about 1-2 hours in Intel core 5.
- Minimum disk space required : 5GB
- fix [Uninitialized warnings](https://gcc.gnu.org/onlinedocs/gcc-10.3.0/gcc/Warning-Options.html), sometime this could be error when the build script sets the flag `-WError` along with `-Wuninitialized` or `-Wmaybe-uninitialized` in gcc, this may happen in several places in the codebase. Try manually fix it without breaking the original logic.

##### Install
- Simply run `make install`
- It's OK to run `make install` without `root` privilege,  then you must ensure that current user account  of your target Linux system has full access permission to the path `CMAKE_INSTALL_PREFIX`.







#### Configuration after Installation (TODO)

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

### Reference
- [MariaDB Dev-team Jira](http://mariadb.org/jira)
* [Get the code, build it, test it](https://mariadb.org/get-involved/getting-started-for-developers/get-code-build-test/)
* [How To Reset Your MySQL or MariaDB Root Password](https://www.digitalocean.com/community/tutorials/how-to-reset-your-mysql-or-mariadb-root-password)
