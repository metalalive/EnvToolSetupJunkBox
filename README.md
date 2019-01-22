### Quick setup for STM32F446 Nucleo board on Ubuntu 14.04 LTS as host OS


### Tool setup
* STM32MXCube, easy to install, download from official webpage
* st-link flash v2, for loading your binary image to flash memory of target board
* install toolchain arm-none-eabi-gcc for cortex-m4 MCU, due to [this issue and its corresponding workaround](https://stackoverflow.com/a/26945980/9853105), DO NOT directly install the toolchain from apt-get repository, instead you should :
  * download arm-none-eabi-gcc (in my case, I downloaded 5.4-2016-q3-update release) from [this site](https://launchpad.net/gcc-arm-embedded/+download)
  * simply extract the downloaded file to /usr/local
  * then add the path to environment variable $PATH
  * type command ```arm-none-eabi-gcc --version``` on terminal to see if it works


### Debug setup
* install gdb-multiarch by ```sudo apt-get install gdb-multiarch``` , it works in my case since I use Ubuntu 14.04 LTS on 64-bit CPU of laptop while STM32F446 Nucleo board includes 32-bit cortex-m4 MCU.

* install latest version of openocd (v0.10.0 at the time I took this note) because the latest version includes configuration scripts for STM32F4 Nucleo board. Please note that the version of openocd on apt-get repository is v0.7.0, which is NOT the latest version, you will have to build the latest version from source.
  * download source from [here](http://openocd.org/2017/01/openocd-0-10-0-release-is-out/)
  * extract the compressed file to ```/PATH/YOU/WANT/openocd/v0.10.0/```
  * ```chmod -R 777 /PATH/YOU/WANT/openocd/v0.10.0/*```
  * ```cd /PATH/YOU/WANT/openocd/v0.10.0/```
  * ```./configure```
  * ```make```
  * ```sudo make install```
  * type ```openocd --version``` to see if it works
  

### Debug Usage
  * open 2 terminals, one is for openocd acted as server, the other one is for gdb-multiarch acted as client
  * ```
    openocd -f /PATH/YOU/WANT/openocd/v0.10.0/tcl/interface/stlink_v2-1.cfg  \
            -f /PATH/YOU/WANT/openocd/v0.10.0/tcl/board/st_nucleo_f4.cfg \
            -c init -c "reset init" 
    ```
  * ```
    gdb-multiarch  /PATH/TO/YOUR_BINARY_IMAGE_ELF_FILE
    ```
    * localhost:3333
    * where; list ;info b;

