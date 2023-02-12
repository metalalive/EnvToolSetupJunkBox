
### Nginx 1.21.6

Environment :
  * Debian 9 (Raspbian Stretch), Ubuntu LTS 14.04
  * OpenSSL 1.1.1c, also built from source


#### Build from source  
configure the build by running `./auto/configure`, carefully check each parameter
```shell
./auto/configure \
 --prefix=/PATH/TO/YOUR/SERVER/SETUP/FOLDER  \
 --user=OS_USER_NAME  --group=OS_USER_GROUP \
 --build=WHATEVER_BUILD_NAME   --builddir=WHATEVER_BUILD_FOLDER_NAME  \
 --with-select_module  --with-poll_module  --with-threads  --with-file-aio \
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
 --with-mail    --with-mail_ssl_module \
 --with-stream \
 --with-stream_ssl_module \
 --with-stream_realip_module \
 --with-stream_ssl_preread_module \
 --with-cpp_test_module \
 --with-compat \
 --with-cc=/PATH/TO/GCC/SPECIFIC_VERSION/BIN/gcc  \
 --with-cc-opt="-g -I/usr/local/include -I/usr/include " \
 --with-ld-opt="-L/usr/local/lib -L/usr/lib "  \
 --with-openssl=/PATH/TO/YOUR/CUSTOM/OPEENSL_HOME \
 --add-module=/PATH/TO/THIRD_PARTY/NGINX_MODULE/HOME  \
 --with-debug  >&  config.log
```

Note :
- Add more paths of headers and libraries to `--with-cc-opt` and `--with-ld-opt`, to successfully complete build procedure
- `--prefix` indicate where your server setup will be located (e.g. configuration files, access log, static files ... etc), once you install, the nginx installer will generate all reqeuired files to `/PATH/TO/YOUR/SERVER/SETUP/FOLDER`
- `--with-debug` is essential if you need to modify logging level of a running nginx instance for debugging purpose.
-  Nginx server will be launched by the user `OS_USER_NAME` in the group `OS_USER_GROUP`, they should be ready in the operating system of server machine before starting the server.
- `WHATEVER_BUILD_FOLDER_NAME` is the working folder for the build , it is generated after `/auto/configure`
- `--with-mail` and `--with-mail_ssl_module` are essential
- you can add `--with-cc` to build nginx using different versions of GCC
- `--add-module` indicates the path to third-party module which will be statically build and linked to nginx executable.


Start building executable
```
make >& build.log
```

Note
- Building nginx takes about 3 hours on Raspberry PI 1
- For openssl v1.1.1 series, make sure the macro parameter `OPENSSL_VERSION_NUMBER` is correct value,  the bit range [31:8] should be `0x101010` . There are commits that contained incorrect / confusing parameters , that will lead to compile error in other C program.

Installation, may be optional
```
sudo make install >& install.log
```

#### Configuration
Here is an example of proxy server, it requires the module [vhost traffic status (nginx-module-vts)](https://github.com/vozlt/nginx-module-vts) built with the nginx server.

```nginx
#user  OS_USER_NAME  OS_USER_GROUP;
worker_processes  1;
## logging level can be `debug`, `info`, `error`
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
    
    ## basic setup for nginx-module-vts
    vhost_traffic_status_zone  shared:vts_stats_shr:3m;
    vhost_traffic_status_filter_by_set_key  $status  resp_status::*;
    vhost_traffic_status_histogram_buckets  0.025 0.05 0.1  0.3  0.5  1 5 10;
    
    # the path has to be present (created by OS_USER_NAME) before starting the server
    proxy_cache_path  customdata/nJeeks/cache  levels=2:2  inactive=4m  use_temp_path=off  max_size=10m  keys_zone=zone_one:2m;

    upstream backend_app_345 {
        server localhost:8010  max_conns=31;
        keepalive   10;
    } # all the identical servers should use the same settings ?




    # you might have another server group running in different setup
    upstream django_backend {
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
} ## end of http block
```
Note:
- [`$status`](http://nginx.org/en/docs/http/ngx_http_core_module.html#var_status) is a built-in variable which indicates response status code.
- `vhost_traffic_status_filter_by_set_key` directive filters the incoming requests by the variable `$status`, the statistic data will be classified by the response status code (e.g. 200, 404, 500). `resp_status::*` is just a label for the report.
- For `proxy_cache_path`
  - `keys_zone` specifies :
    - the label of the **zone** where cached data is stored. Different server contexts / blocks may share the same **zone**.
    - size of preserved space (in MBytes) used for the keys in **zone**. One key is mapped to one cached file.
  - the absolute path of cache will be `/PATH/TO/YOUR/SERVER/SETUP/FOLDER/customdata/nJeeks/cache`
  - `levels` indicates local path to cached data under `customdata/nJeeks/cache`
  - `inactive` specifies how many minutes a cached data will be stored, that is, Nginx automatically deletes the cached file which hasn't been accessed over `inactive` minutes (in this example, it is 4 minutes).
  - `max_size` size of preserved space (in MBytes) limited to the current cache zone.

#### Run
Go to `/PATH/TO/YOUR/SERVER/SETUP_FOLDER` run the command :

```
./sbin/nginx -c  ./conf/nginx.conf
```
- if your `nginx.conf` is not in `/PATH/TO/YOUR/SERVER/SETUP_FOLDER/conf`, you will need to specify full path of the configuration file.
- once syntax check of the configuration file passed, nginx will run as a daemon if `pid` is set in `nginx.conf`
- From v1.21, this command requires superuser privilege to execute.

For reloading nginx configurations without stopping the running instance, run the command:
```
./sbin/nginx -s reload -c  ./conf/nginx.conf
```

The command below will (gracefully ?) terminate the running nginx instance :
```
./sbin/nginx -s stop -c  ./conf/nginx.conf
```
alternatively you can kill the process of nginx instance by the process ID
```
cat  logs/nginx.pid
sudo kill -SIGTERM  CURR_NGINX_PID
```

#### Reference
* [Build Nginx from source -- MatthewVance](https://github.com/MatthewVance/nginx-build/blob/master/build-nginx.sh)
* [How To Compile Nginx From Source and Install on Raspbian Jessie](https://www.linuxbabe.com/raspberry-pi/compile-nginx-source-raspbian-jessie)
* [Build Nginx from source - offiical doc](https://docs.nginx.com/nginx/admin-guide/installing-nginx/installing-nginx-open-source/)
* [Configure Nginx running with uWSGI](https://uwsgi-docs.readthedocs.io/en/latest/tutorials/Django_and_nginx.html#basic-nginx)
* [How To Set Up uWSGI and Nginx to Serve Python Apps on Ubuntu 14.04](https://www.digitalocean.com/community/tutorials/how-to-set-up-uwsgi-and-nginx-to-serve-python-apps-on-ubuntu-14-04)
* [Nginx full-example configuration](https://www.nginx.com/resources/wiki/start/topics/examples/full/)

