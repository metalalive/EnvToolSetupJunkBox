
### Nginx 1.17.9

Environment :
  * Debian 9 (Raspbian Stretch), Ubuntu LTS 14.04
  * OpenSSL 1.1.1c, also built from source


#### Build from source  
configure the build by running `./auto/configure`, carefully check each parameter
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
 --with-cc-opt="-g -I/usr/local/include -I/usr/include " \
 --with-ld-opt="-L/usr/local/lib -L/usr/lib "  \
 --with-openssl=/PATH/TO/YOUR/CUSTOM/OPEENSL_HOME \
 --with-debug  >&  config.log

```

Note that :
* you may need to add more paths to `--with-cc-opt` and `--with-ld-opt` successfully complete `cmake` procedure
* `--prefix` indicate where your server setup will be (e.g. configuration files, log files, static files ... etc), once you install nginx by running `make install` the nginx installer will generate all reqeuired files to `/PATH/TO/YOUR/SERVER/SETUP_FOLDER`, that is, the folder specified in `--prefix` option
* `--with-debug` is essential if you need to modify logging level of a running nginx instance for debugging purpose.


Building nginx takes about 3 hours on Raspberry PI 1
```
make >& build.log &
```

Installation, may be optional
```
sudo make install >& install.log &
```

#### Configuration
Here is a simple example :

```perl
#user  nobody; # will be www-data
worker_processes  1;
# nginx seems to log error messages at INFO level
error_log   logs/nginx.error.log info;
pid         logs/nginx.pid; # to run nginx instance as a daemon
worker_rlimit_nofile  128;

events {
    worker_connections  5; # must not be greater than worker_rlimit_nofile
}

http {
    include       conf/mime.types;
    default_type  application/octet-stream;
    sendfile      on;
    keepalive_timeout  37;

    # the upstream component nginx server needs to connect to
    upstream django {
        server localhost:8009; # use web port socket
    }

    server {
        listen       8008;
        server_name  localhost;
        charset      utf-8;
        access_log   logs/nginx.access.log;
        client_max_body_size  16M;
        allow        127.0.0.1; # allow the requests from these IPs (they cannot be domain name)
        deny         all; # deny rest of IPs, note the order of the allow/deny declaratives will affect your IP whitelisting

        location / { # forward all the paths to WSGI application (hosted in uwsgi in this case)
            include     conf/uwsgi_params;
            uwsgi_pass  django; # specify the name of upstream component
            ##uwsgi_pass   localhost:8009;
        }
    }
}
```

#### Run
Go to `/PATH/TO/YOUR/SERVER/SETUP_FOLDER` run the command :

```
./sbin/nginx -c  ./conf/nginx.conf
```
* if your `nginx.conf` is not in `/PATH/TO/YOUR/SERVER/SETUP_FOLDER/conf`, you will need to specify full path of the configuration file.
* once syntax check of the configuration file passed, nginx will run as a daemon if `pid` is set in `nginx.conf`

For reloading nginx configurations without stopping the running instance, run the command:
```
./sbin/nginx -s reload -c  ./conf/nginx.conf
```

The command below will (gracefully ?) terminate the running nginx instance :
```
./sbin/nginx -s stop -c  ./conf/nginx.conf
```

#### Reference
* [Build Nginx from source -- MatthewVance](https://github.com/MatthewVance/nginx-build/blob/master/build-nginx.sh)
* [How To Compile Nginx From Source and Install on Raspbian Jessie](https://www.linuxbabe.com/raspberry-pi/compile-nginx-source-raspbian-jessie)
* [Build Nginx from source - offiical doc](https://docs.nginx.com/nginx/admin-guide/installing-nginx/installing-nginx-open-source/)
* [Configure Nginx running with uWSGI](https://uwsgi-docs.readthedocs.io/en/latest/tutorials/Django_and_nginx.html#basic-nginx)
* [How To Set Up uWSGI and Nginx to Serve Python Apps on Ubuntu 14.04](https://www.digitalocean.com/community/tutorials/how-to-set-up-uwsgi-and-nginx-to-serve-python-apps-on-ubuntu-14-04)
* [Nginx full-example configuration](https://www.nginx.com/resources/wiki/start/topics/examples/full/)

