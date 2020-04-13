
#### Nginx 1.17.9

###### Build from source
* Environment :
  * Debian 9 (Raspbian Stretch)
  * OpenSSL 1.1.1c, also built from source
  
* configure the build by running `./auto/configure`, carefully check each parameter
```
./auto/configure \
 --prefix=/PATH/TO/YOUR/SERVER/SETUP_FOLDER  \
 --user=webuser  --group=webuser \
 --build=nginxhttpsite \
 --with-threads \
 --with-file-aio \
 --with-http_ssl_module \
 --with-http_v2_module \
 --with-http_realip_module  \
 --with-http_addition_module \
 --with-http_sub_module \
 --with-http_dav_module \
 --with-http_flv_module \
 --with-http_mp4_module \
 --with-http_gunzip_module \
 --with-http_gzip_static_module \
 --with-http_auth_request_module \
 --with-http_random_index_module \
 --with-http_secure_link_module \
 --with-http_slice_module \
 --with-http_stub_status_module \
 --with-mail \
 --with-mail_ssl_module \
 --with-stream \
 --with-stream_ssl_module \
 --with-stream_realip_module \
 --with-stream_ssl_preread_module \
 --with-cpp_test_module \
 --with-compat \
 --with-cc-opt="-g -I/usr/local/include" \
 --with-ld-opt="-L/usr/local/lib"  \
 --with-openssl=/PATH/TO/YOUR/CUSTOM/OPEENSL_HOME \
 --with-debug  >&  config.log

```

* Building nginx takes about 3 hours on Raspberry PI 1
```
make >& build.log &
```


* Installation, may be optional
```
sudo make install >& install.log &
```



##### Reference
* [Build Nginx from source -- MatthewVance](https://github.com/MatthewVance/nginx-build/blob/master/build-nginx.sh)
* [How To Compile Nginx From Source and Install on Raspbian Jessie](https://www.linuxbabe.com/raspberry-pi/compile-nginx-source-raspbian-jessie)

