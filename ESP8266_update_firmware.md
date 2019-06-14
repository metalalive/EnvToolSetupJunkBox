#### Firmware source
Here are sources you can get pre-built firmware or source code which are required to be built using cross-compiler:
* ESP Non-OS SDK, provide pre-built firmware binary
* ESP RTOS SDK,  download the source then build binary by yourself.
* ESP-IDF, new integrated tool to flash firmware, officially recommended

#### Download firmware to target ESP8266 board

* prepare USB-to-TTL cable connecting Tx/Rx pins of ESP8266 board (in my case it's ESP-01s) and host PC which runs ```esptool``` later.
* wire our ESP-01s with USB-to-TTL serial cable as below :

| pin of ESP-01   | other devices |
|-----------------|---------------|
| VDD, CH_PD  |  3.3 Volt. (we use 3.3V pin of STM32 board as power supply) |
| GND, GPIO0        | GND of the power supply    |
| TX                | RX of USB-TTL serial cable |
| RX                | TX of USB-TTL serial cable |
| GPIO2             | left unconnected |


* Download pre-built ESP8266 non-OS SDK . From version 3.0.0, ESP8266 Non-OS SDK no longer provides firmware for ESP device with flash size smaller than 2 MB (8Mb), therefore we used the version v2.2.0 of ESP non-OS SDK for ESP-01s because it provides firmware for flash size 1MB.
* Downlaod esptool, install required package for python3 (python2 doesn't work in my case), [see the issue HERE](https://github.com/espressif/esptool/issues/324)
* perform hardware reset before running any of esptool commands.
* check flash size of the ESP device first. 
  ```
  python3 ../esptool/esptool.py  --port /dev/ttyUSB0  flash_id
  ```
  then get response below
  ```
  esptool.py v2.6
  Serial port /dev/ttyUSB0  
  Connecting....
  Detecting chip type... ESP8266
  Chip is ESP8266EX
  Features: WiFi
  MAC: 5c:cf:7f:ff:72:ba
  Uploading stub...
  Running stub...
  Stub running...
  Manufacturer: e0
  Device: 4014
  Detected flash size: 1MB
  Hard resetting via RTS pin...
  ```
  
* program the prebuilt firmware using ```esptool``` , the command :
  ```
  sudo python3  YOUR_PATH_TO_ESPTOOL/esptool.py -p /dev/ttyUSB0 -b 115200 write_flash \
                0x0000 ./bin/boot_v1.6.bin \
                0x1000 ./bin/at/512+512/user1.1024.new.2.bin \
                0xfc000 ./bin/esp_init_data_default_v08.bin \
                0x7e000 ./bin/blank.bin \
                0xfe000 ./bin/blank.bin 
  ```

when you get result like following, the firmware should be successfully written to target board.
```
esptool.py v2.6
Serial port /dev/ttyUSB0
Connecting....
Detecting chip type... ESP8266
Chip is ESP8266EX
Features: WiFi
MAC: 5c:cf:7f:ff:72:ba
Uploading stub...
Running stub...
Stub running...
Configuring flash size...
Auto-detected Flash size: 1MB
Flash params set to 0x0020
Compressed 3856 bytes to 2762...
Wrote 3856 bytes (2762 compressed) at 0x00000000 in 0.2 seconds (effective 123.6 kbit/s)...
Hash of data verified.
Compressed 407796 bytes to 293037...
Wrote 407796 bytes (293037 compressed) at 0x00001000 in 26.2 seconds (effective 124.6 kbit/s)...
Hash of data verified.
Compressed 128 bytes to 75...
Wrote 128 bytes (75 compressed) at 0x000fc000 in 0.0 seconds (effective 95.6 kbit/s)...
Hash of data verified.
Compressed 4096 bytes to 26...
Wrote 4096 bytes (26 compressed) at 0x0007e000 in 0.0 seconds (effective 5502.8 kbit/s)...
Hash of data verified.
Compressed 4096 bytes to 26...
Wrote 4096 bytes (26 compressed) at 0x000fe000 in 0.0 seconds (effective 5545.7 kbit/s)...
Hash of data verified.

Leaving...
Hard resetting via RTS pin...
```

#### Reference
* [ESP8266 01 overview and flashing instructions](https://github.com/jandelgado/NodeMCU/wiki/ESP8266-01-overview-and-flashing-instructions)
* [How to Flash ESP-01 Firmware to the Improved SDK v2.0.0](https://www.allaboutcircuits.com/projects/flashing-the-ESP-01-firmware-to-SDK-v2.0.0-is-easier-now/)
* [NodeMCU Flashing the firmware](https://nodemcu.readthedocs.io/en/master/flash/)
