
### Paho
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

### Mosquitto
* Mosquitto publish command sample
```
mosquitto_pub -h 123.44.5.67 -p 8883 -u USERNAME -P PASSWD  -i PUB_ID  -q 2 --tls-version tlsv1.3  -V mqttv5 \
    --cafile ./ca_crt.pem --cert ./ca_crt.pem  --key ./ca_priv_key.pem  \
    -d  -t  YOUR_TOPIC_STRING  -m "YOUR-MESSSAGE_CONTENT_IS_HERE"
```
  users can append extra options `-r` (retain message enabled)
  

* Mosquitto subscribe command sample : as the same as the publish command below, without message payload option `-m `

* Create new user (and corresponding password) to Mosquitto broker
```
// create new password file PATH_TO_PASSWD_FILE and add new user/passwd in
mosquitto_passwd -c PATH_TO_PASSWD_FILE  NEW_USER_NAME
// append new new user/passwd to existing PATH_TO_PASSWD_FILE
mosquitto_passwd    PATH_TO_PASSWD_FILE  NEW_USER_NAME
```

