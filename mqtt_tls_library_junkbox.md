
### Paho C MQTT
Paho install instruction
```
make PAHO_WITH_SSL=TRUE PAHO_BUILD_DOCUMENTATION=FALSE
sudo  make install  PAHO_WITH_SSL=TRUE PAHO_BUILD_DOCUMENTATION=FALSE
```


* compile libtommath

```
make clean; make libtommath.a V=1
make test
```

* compile libtomcrypt
```
make clean; make  V=1  EXTRALIBS="../libtommath/libtommath.a" CFLAGS="-I../libtommath"
make test EXTRALIBS="../libtommath/libtommath.a"
```

### libwebsockets for mosquitto MQTT
For those who need to build from source, clone the source from github, go to the source folder.
```
mkdir build
cd build
```
For those who also build openssl from source, you have:
```
cmake .. -DOPENSSL_ROOT_DIR=/usr/local   -DCMAKE_INCLUDE_DIRECTORIES_PROJECT_BEFORE=/usr/local  \
    -DLWS_OPENSSL_INCLUDE_DIRS=/usr/local/include/openssl  \
    -DLWS_OPENSSL_LIBRARIES="/usr/local/lib/libssl.so;/usr/local/lib/libcrypto.so"  \
    -DCMAKE_BUILD_TYPE=DEBUG   -DLWS_WITH_HTTP2=1   -DLWS_ROLE_MQTT=1  >&  cmake.log

make

sudo make install
```
check `libwebsockets.so` in `/usr/local/lib`, it should be linked to the version you just installed.


### Mosquitto MQTT broker / client
* Build from source
```
make WITH_DOCS=no  CPPFLAGS="-DSSL_OP_NO_TLSv1_3"  WITH_WEBSOCKETS=yes

sudo make install  WITH_DOCS=no  WITH_WEBSOCKETS=yes
```

* Mosquitto publish command sample
```
mosquitto_pub -h 123.44.5.67 -p 8883 -u USERNAME -P PASSWD  -i PUB_ID  -q 2 --tls-version tlsv1.3  -V mqttv5 \
    --cafile ./ca_crt.pem --cert ./ca_crt.pem  --key ./ca_priv_key.pem  \
    -d  -t  YOUR_TOPIC_STRING  -m "YOUR-MESSSAGE_CONTENT_IS_HERE"
```
  users can append extra options `-r` (retain message enabled)
  

* Mosquitto subscribe command sample : as the same as the publish command above, without message payload option `-m `

* Create new user (and corresponding password) to Mosquitto broker
```
// create new password file PATH_TO_PASSWD_FILE and add new user/passwd in
mosquitto_passwd -c PATH_TO_PASSWD_FILE  NEW_USER_NAME
// append new new user/passwd to existing PATH_TO_PASSWD_FILE
mosquitto_passwd    PATH_TO_PASSWD_FILE  NEW_USER_NAME
```

* ACL suppport, according to [this issue](https://github.com/mqttjs/MQTT.js/issues/714) and [this](https://github.com/eclipse/mosquitto/issues/803#issuecomment-386110952) , ACL with `Mosquitto MQTT broker` may not prevent subscription. Subscriptions that are considered invalid are NEVER denied at `Mosquitto MQTT broker` (v1.6.9, in my case). (they should be)
```
# example below
user someadmin
topic write  topic1/ctrl/+
topic read   topic2/log/+ 
 
user username123 
topic write  topic1/log/username123 
topic read   topic2/ctrl/username123 
 
user username456 
topic write  topic1/log/username456 
topic read   topic2/ctrl/username456 

// For example, even in mosquitto broker v1.6.9 with this ACL file, username123 can still subscribe ANY topic,
// but publish with ACL works as expected, username123 is restricted to publish with topic1/log/username123
// after verification.
```

