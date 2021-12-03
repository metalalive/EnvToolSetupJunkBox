#### Usage

* generate makefile
```
mkdir -p build
cd build
rn -rf ./*
GCC_INSTALL_DIR="/PATH/TO/GCC/BINNARY_FOLDER" PKG_CONFIG_PATH="/PATH/TO/libuv_project_home"  cmake ..
```
* generate executable
```
make clean; make <OPTIONAL_EXECUTABLE_FILENAME_DEFINED_IN_CMAKE>;
```

* Run
```
rm -rf /PATH/TO/DST_FILE;
/PATH/TO/EXECUTABLE  /PATH/TO/SRC_FILE  /PATH/TO/DST_FILE
```

* Debug
```
rm -rf /PATH/TO/DST_FILE;
gdb --args  /PATH/TO/EXECUTABLE  /PATH/TO/SRC_FILE  /PATH/TO/DST_FILE
```

* Memory Test
```
rm -rf /PATH/TO/DST_FILE;
valgrind --leak-check=full /PATH/TO/EXECUTABLE  /PATH/TO/SRC_FILE  /PATH/TO/DST_FILE
```

