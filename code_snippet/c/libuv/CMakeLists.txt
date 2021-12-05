CMAKE_MINIMUM_REQUIRED(VERSION 2.8.0)
# should be set before project() command
SET(CMAKE_C_COMPILER   "$ENV{GCC_INSTALL_DIR}/bin/gcc")
SET(CMAKE_CXX_COMPILER "$ENV{GCC_INSTALL_DIR}/bin/g++")
SET(CMAKE_C_FLAGS_DEBUG  "-O0")
SET(CMAKE_C_FLAGS_RELEASE  "-O2")

PROJECT(learn_libuv C)

INCLUDE(GNUInstallDirs)
INCLUDE(CheckCSourceCompiles)
INCLUDE(CMakePushCheckState)
INCLUDE(CTest)

FIND_PACKAGE(PkgConfig)
FIND_PACKAGE(Threads REQUIRED)

CHECK_C_SOURCE_COMPILES("
#include <stdint.h>
int main(void) {
uint64_t a;
__sync_add_and_fetch(&a, 1);
return 0;
}" ARCH_SUPPORTS_64BIT_ATOMICS)

CMAKE_PUSH_CHECK_STATE()
SET(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -pthread")
CHECK_C_SOURCE_COMPILES("
#define _GNU_SOURCE
#include <sched.h>
#include <pthread.h>
int main(void) {
#ifndef __NetBSD__
cpu_set_t s;
CPU_ZERO(&s);
pthread_setaffinity_np(0, sizeof(cpu_set_t), &s);
#else
cpuset_t *s = cpuset_create();
cpuset_zero(s);
pthread_setaffinity_np(0, cpuset_size(s), s);
cpuset_destroy(s);
#endif
return 0;
}" HAS_PTHREAD_SETAFFINITY_NP)
CMAKE_POP_CHECK_STATE()

# specify path to pkg-config metadata file for libuv in the environment variable PKG_CONFIG_PATH
IF (PKG_CONFIG_FOUND)
    PKG_CHECK_MODULES(LIBUV libuv>=1.0.0)
    # if found, the variables will be automatically set by cmake :
    # * LIBUV_INCLUDE_DIRS
    # * LIBUV_LIBRARY_DIRS
    # * LIBUV_LIBRARIES
    IF (LIBUV_FOUND)
        INCLUDE_DIRECTORIES(${LIBUV_INCLUDE_DIRS})
        LINK_DIRECTORIES(${LIBUV_LIBRARY_DIRS})
    ENDIF (LIBUV_FOUND)
ENDIF (PKG_CONFIG_FOUND)

#MESSAGE("check out LIBUV_INCLUDE_DIRS ${LIBUV_INCLUDE_DIRS}")
#MESSAGE("check out LIBUV_LIBRARY_DIRS ${LIBUV_LIBRARY_DIRS}")
#MESSAGE("check out LIBUV_LIBRARIES ${LIBUV_LIBRARIES}")

# add debug info to executable
SET(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -g")
# should be set after link_directory() command
ADD_EXECUTABLE(file_access.out file_access.c)
ADD_EXECUTABLE(pipe_stream.out pipe_stream.c)

TARGET_LINK_LIBRARIES(file_access.out ${LIBUV_LIBRARIES})
TARGET_LINK_LIBRARIES(pipe_stream.out ${LIBUV_LIBRARIES})

