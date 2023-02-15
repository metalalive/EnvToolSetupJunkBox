### Use cases
- reverse proxy
- content caching (e.g. popular PDF files, video/audio segments)
- Load balancing

Relevant background knowledge:
- [Proxy](https://security.stackexchange.com/questions/189088)
  - something acting on behalf of something else;
    - the 3rd computer (say `Px`) sits in between 2 computers (say `A1` and `A2`).
    - for example : `A1` requests a file in `A2`
  - normally, `A1` and `A2` communicate directly with each other without `Px`
  - in some cases, proxy `Px` is a better option :
    - some files are so popular, large traffic volume is flooding in to `A2`
    - too many files to serve in one single computer `A2`
    - security concern, `A1` doesn't know about `A2` (trusted or malicious) and vice versa.
- [Forward proxy vs. reverse proxy (StackOverflow)](https://stackoverflow.com/questions/224664)
  - forward proxy:
    - `Px` acts on behalf of `A1` and talks to `A2`. `A2` doesn't know `A1`
    - use case : some administrative authority (`Px`) controls `A1`'s internet access, so `A1` cannot directly talk to `A2`
  - reverse proxy:
    - `Px` acts on behalf of `A2` and talks to `A1`. `A1` doesn't know `A2`
    - use case : 
      - cache popular files from `A2` to `Px`, then `A1` requests a popular file in `Px`, this can reduce:
        - traffic volume to `A2`
        - response time of the file
      - add new computer (say `A20`) to serve some files and work in parallel with `A2`.
        - `A1` is NOT aware of 2 different computers serving files, since it only talks to `Px`

![diagram of proxying](https://i.stack.imgur.com/0qpxZ.png)

### Nginx version
1.21.6

### Build from source
Environment :
  * Debian 9 (Raspbian Stretch), Ubuntu LTS 14.04
  * OpenSSL 1.1.1c, also built from source

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

### Configuration
Here is an example of reverse proxy server, it requires the module [vhost traffic status (nginx-module-vts)](https://github.com/vozlt/nginx-module-vts) built with the nginx server.

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
        # could scale to serveral app servers
        server localhost:8010  max_conns=31;
        server localhost:8011  max_conns=38;
        keepalive   10;
    } # all the servers at here should use the same settings ?
    
    server { ## start of proxy server block
        listen       8050  ssl;
        server_name  localhost;
        error_page   404 410              /404.html;
        error_page   500 502 503 504  515 /50x.html;
        ssl_certificate      /PATH/TO/PROXY/SERVER.crt;
        ssl_certificate_key  /PATH/TO/PROXY/SERVER/secret.key;
        
        location /status {
            vhost_traffic_status_display;
            vhost_traffic_status_display_format html;
        } ## TODO, authorization

        # Debug log file shows Nginx failed to open local file `/html/file`
        # then immediately return 404 without pass the request to backend app.
        # Regex location might not be able to work with proxy_pass directive ? (TODO)
        # location ~ ^/file\?res_id=[A-Za-z0-9%]+$ {
        location  /file {
            proxy_pass  https://backend_app_345;
            proxy_ssl_protocols   TLSv1.3;  # prerequisites, openssl >= v1.1.1
            proxy_ssl_certificate      /PATH/TO/PROXIED/UPSTREAM/ca.crt;
            proxy_ssl_certificate_key  /PATH/TO/PROXIED/UPSTREAM/ca.secret.key;
            # added if it is self-signed CA (not well-known trusted CA).
            proxy_ssl_trusted_certificate  /PATH/TO/PROXIED/UPSTREAM/ca.crt;
            ##proxy_ssl_verify  on; ## error on client (proxy server) verify
            proxy_http_version    1.1;      # http/2 still not supported, figure out why (TODO)
            add_header  X-Cache-Status  $upstream_cache_status;
            proxy_cache      zone_one;
            proxy_cache_key  $request_uri;  # can also use custom header from upstream backend
            proxy_cache_valid  200  1m;
            proxy_no_cache   $WHATEVER_VARIABLE;
            proxy_buffering  on; # it is turned on by default
            proxy_buffer_size  5k;
            proxy_buffers    7  9k;
        }
    } # end of proxy server block


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
            uwsgi_pass  django_backend; # specify the name of upstream component
            ##uwsgi_pass   localhost:8009;
        }
    }
} ## end of http block
```

#### Note
- [`$status`](http://nginx.org/en/docs/http/ngx_http_core_module.html#var_status) is a built-in variable which indicates response status code.
- the endpoint `/status` of the proxy server provides statistic data in HTML format, there is also JSON format generated by another endpoint `/status/format/json`.
- `vhost_traffic_status_filter_by_set_key` directive filters the incoming requests by the variable `$status`, the statistic data will be classified by the response status code (e.g. 200, 404, 500). `resp_status::*` is just a label for the report.
- `ssl_certificate` and `ssl_certificate_key` is acceptable ONLY in server block, in this example the certificate is applied between frontend client and the proxy server.


### Reverse Proxy in Nginx
According to the configuration above, there are key factors which build up proxy server:

- `proxy_ssl_certificate` and `proxy_ssl_certificate_key` are the cert and key respectively applied between the proxy server and upstream backend app server.
- In `proxy_pass`, you can also specify upstream label (in this example, it is `backend_app_345`) , which passes the request to one of the sevrers in the upstream block.
##### Caching
- Use case : popular response, not frequently changed / modified
- `proxy_cache_path`
  - `keys_zone` specifies :
    - the label of the **zone** where cached data is stored. Different server contexts / blocks may share the same **zone** (in this example, it is `zone_one`).
    - size of preserved space (in MBytes) used for the keys in **zone**. One key is mapped to one cached file.
  - the absolute path of cache will be `/PATH/TO/YOUR/SERVER/SETUP/FOLDER/customdata/nJeeks/cache`
  - `levels` indicates local path to cached data under `customdata/nJeeks/cache`
  - `inactive` specifies how many minutes a cached data will be stored, that is, Nginx automatically deletes the cached file which hasn't been accessed over `inactive` minutes (in this example, it is 4 minutes).
  - `max_size` size of preserved space (in MBytes) limited to the current cache zone.
- `proxy_cache_key` specifies the key to the cache zone `zone_one`. Note that the key can NOT be used directly to look for final cached file saved in `customdata/nJeeks/cache`.
- To cache a file, a cache zone internally generate another key in MD5 hash format then use it as the path to the cached file under `customdata/nJeeks/cache`
- `proxy_cache_valid` indicates expiry time (in minute) if an upstream server responds with expected status code.
  - In this example, response body from upstream server is cached for `/file` endpoint, if the status code is `200` (ok)
  - For any request to access an existing cached file:
    - if the time is less than 1 minute after the file was saved, the proxy server simply returns the cached file. 
    - otherwise, the file is expired, the proxy server sends a request again to upstream server for content update.
  - The header [`cache-control`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) takes precedence over `proxy_cache_valid` if it is present in the response of upstream app server.
    - if directive `max-age` in `cache-control` is set to positive number, then nginx caches the response body.
    - otherwise, if directives `no-cache`, `private` are specified in the header, nginx will NOT cache the response body.
- `proxy_no_cache` makes nginx NOT cache the response if `$WHATEVER_VARIABLE` is either an non-empty string or non-zero number,
- For `proxy_cache_valid` vs. `inactive` in `proxy_cache_path`, see [this StackOverflow thread](https://stackoverflow.com/questions/64151378/).
##### Buffering
- Use case: Slow client downloading chunks of response data (synchronously) from the proxy server
- `proxy_buffering`, turned on by default
- the size argument in `proxy_buffers` and `proxy_buffer_size` defaults to memory page size of the OS (4KB | 8KB)
- `proxy_buffer_size` specifies the size only for first chunk of response ; while `proxy_buffers` specifies the size for subsequent chunks 

#### TODO
- Regex location might not be able to work with `proxy_pass` directive ?
- if `no-cache` is present in the header `cache-control`, does `proxy_no_cache` still take effect ?

### Load Balancing in Nginx
#### Layer 7 (HTTP)
#### Layer 4 (TCP/UDP)

### Run
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

### Reference
* [Build Nginx from source -- MatthewVance](https://github.com/MatthewVance/nginx-build/blob/master/build-nginx.sh)
* [How To Compile Nginx From Source and Install on Raspbian Jessie](https://www.linuxbabe.com/raspberry-pi/compile-nginx-source-raspbian-jessie)
* [Build Nginx from source - offiical doc](https://docs.nginx.com/nginx/admin-guide/installing-nginx/installing-nginx-open-source/)
* [Configure Nginx running with uWSGI](https://uwsgi-docs.readthedocs.io/en/latest/tutorials/Django_and_nginx.html#basic-nginx)
* [How To Set Up uWSGI and Nginx to Serve Python Apps on Ubuntu 14.04](https://www.digitalocean.com/community/tutorials/how-to-set-up-uwsgi-and-nginx-to-serve-python-apps-on-ubuntu-14-04)
* [Nginx full-example configuration](https://www.nginx.com/resources/wiki/start/topics/examples/full/)
* [A Guide to Caching with NGINX and NGINX Plus](https://www.nginx.com/blog/nginx-caching-guide/)
* [Proxy vs. Reverse Proxy (Explained by Example)](https://www.youtube.com/watch?v=ozhe__GdWC8)
* [Nginx HTTP load balancing](https://docs.nginx.com/nginx/admin-guide/load-balancer/http-load-balancer/)
* [ServerFault - NGINX proxy cache time with Cache-Control](https://serverfault.com/questions/915463)

