### MariaDB setup (build from source, and configure)

##### Envoronment
* Ubuntu 14.04LTS , Debian 9 Stretch (Raspbian Stretch)
* OpenSSL 1.1.1c, built from source
* MariaDB server 10.3.22

##### Pre-requisite
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
```
cmake ..  -LH  -DBUILD_CONFIG=mysql_release   -DCMAKE_BUILD_TYPE=Debug \
    -DWITH_SSL=/PATH/TO/YOUR/OPENSSL/SRC/FOLDER  \
    -DCMAKE_INSTALL_PREFIX=/PATH/TO/INSTALLED/MARIADB/FOLDER  \
    -DWITH_UNIT_TESTS=OFF \
    >& cmake.log &
```


make ..... a lot of problems

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
