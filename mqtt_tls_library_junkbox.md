

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


* Mosquitto publish command sample
```
mosquitto_pub -h 123.44.5.67 -p 1883 -u USERNAME -P PASSWD  -i PUB_ID  -q 2 --tls-version tlsv1.3  -V mqttv5 --cafile ./ca_crt.pem --cert ./ca_crt.pem  --key ./ca_priv_key.pem  -d  -t  YOUR_TOPIC_STRING            -m "If ng to duplication on the server system."

```
