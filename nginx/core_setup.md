### Use cases
- [reverse proxy](https://hackmd.io/@0V3cv8JJRnuK3jMwbJ-EeA/SJF5JD8Ao#Proxy)
- content caching (e.g. popular PDF files, video/audio segments)
- [Load balancing](https://hackmd.io/@0V3cv8JJRnuK3jMwbJ-EeA/SJF5JD8Ao#Load-balancing)

### Nginx version
1.23.3

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
Here is an example of reverse proxy server, I put all directives into one single file. It also requires extra module [vhost traffic status (nginx-module-vts)](https://github.com/vozlt/nginx-module-vts) built with the nginx server.

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
    keepalive_timeout  25;
    
    ## basic setup for nginx-module-vts
    vhost_traffic_status_zone  shared:vts_stats_shr:3m;
    vhost_traffic_status_filter_by_set_key  $status  resp_status::*;
    vhost_traffic_status_histogram_buckets  0.025 0.05 0.1  0.3  0.5  1 5 10;
    
    # the path has to be present (created by OS_USER_NAME) before starting the server
    proxy_cache_path  customdata/nJeeks/cache  levels=2:2  inactive=4m  use_temp_path=off  max_size=10m  keys_zone=zone_one:2m;
    proxy_cache_lock_timeout  11s;
    proxy_cache_lock_age      10s;
    proxy_read_timeout        53s;
    proxy_send_timeout        47s;
    
    limit_conn_zone  $binary_remote_addr  zone=mylmt_conns_state:1m;
    limit_req_zone   $binary_remote_addr  zone=mylmt_reqs_state:2m rate=3r/s;

    upstream backend_app_345 {
        # could scale to serveral app servers
        server localhost:8010  max_conns=31  max_fails=3 fail_timeout=7s  weight=3;
        server localhost:8011  max_conns=38  max_fails=3 fail_timeout=8s  weight=5;
        keepalive   10;
    } # all the servers at here should use the same settings ?
    
    server { ## start of proxy server block
        listen       8050  ssl;
        server_name  localhost;
        error_page   404 410              /404.html;
        error_page   500 502 503 504  515 /50x.html;
        ssl_certificate      /PATH/TO/PROXY/SERVER.crt;
        ssl_certificate_key  /PATH/TO/PROXY/SERVER/secret.key;
        ##ssl_session_ticket_key   /PATH/TO/SESSION-TICKET.key;
        ssl_session_cache  shared:ALIAS_OF_THE_CACHE:3m;
        ssl_protocols   TLSv1.3  TLSv1.2;
        ssl_session_timeout   6m;
        
        location ~ \.jpg$ {
            limit_rate   17k;
        }
        
        location /status {
            vhost_traffic_status_display;
            vhost_traffic_status_display_format html;
            limit_req  zone=mylmt_reqs_state  burst=2;
            limit_req_status  430;
            allow  123.4.56.78;
            allow  127.0.0.1;
            deny   all;
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
            limit_conn  mylmt_conns_state  1;
            limit_conn_status  429;
            proxy_cache_lock  on;
            proxy_cache_use_stale   error timeout updating;
        } ## end of location block
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
- once `ssl_session_ticket_key` is specified with path of pre-shared key file, application will take responsibility to rotate the key, Nginx **won't** do that automatically.
- `ssl_session_cache` indicate the cache area for session storage, for TLS handshake ooptimization (???????)
  - for example in TLS 1.3, the cache internally saves psk (pre-shared keys) for sessions which hasn't expired yet
  - to verify whether TLS 1.3 and its pre-shared key functionality is up and running, use openssl utility [`s_client`](../openssl_cmd_note.md#tls-connection) instead of [`curl`](../curl.md), because `curl` doesn't seem to provide command option for session resumption (e.g. fetch pre-shared key data in previous `curl` command and then specify it in next `curl` command, also `--sessionid` doesn't exist in `curl` command)
- `ssl_session_timeout` in TLS 1.3 indicates lifetime of each cached session (note nginx sends [`NewSessionTicket`](https://www.rfc-editor.org/rfc/rfc8446#section-4.6.1) message after handshake is done successfully)


### Reverse Proxy in Nginx
According to the configuration above, there are key factors which build up proxy server:

- `proxy_ssl_certificate` and `proxy_ssl_certificate_key` are the cert and key respectively applied between the proxy server and upstream backend app server.
- In `proxy_pass`, you can also specify upstream label (in this example, it is `backend_app_345`) , which passes the request to one of the sevrers in the [upstream block](https://nginx.org/en/docs/http/ngx_http_upstream_module.html#upstream).
- `proxy_read_timeout` and `proxy_send_timeout` indicates timeout seconds (in the example above, `53` and `47`) between 2 successive reads / writes respectively at layer 4 packet (TCP / UDP) [(reference)](https://stackoverflow.com/q/70824741/9853105)
#### Caching
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
#### Buffering
- Use case: Slow client downloading chunks of response data (synchronously) from the proxy server
- `proxy_buffering`, turned on by default
- the size argument in `proxy_buffers` and `proxy_buffer_size` defaults to memory page size of the OS (4KB | 8KB)
- `proxy_buffer_size` specifies the size only for first chunk of response ; while `proxy_buffers` specifies the size for subsequent chunks
#### Lock
- `proxy_cache_lock` enables / disables lock on a cache element with the key specified according to `proxy_cache_key`, that ensures:
  - When several reqeusts attempt to populate (create) a new cache element
    - there's only one of them (say the 1st request) succeeded to do so, and pass the request to origin server
    - the others have to wait (*maybe blocking*) for the cache element created / ready.
  - timeout might occur after few seconds specified in `proxy_cache_lock_timeout` or `proxy_cache_lock_age`
- `proxy_cache_lock_timeout`, on occurrence of the timeout:
  - nginx passes the 2nd request, and **NOT cache the response of the 2nd request**
  - it still waits for caching response of the 1st request
- `proxy_cache_lock_age`, on occurrence of the timeout:
  - nginx passes the 2nd request and **cache the response of the 2nd request**
- For example, assume `limit_conn = 10`, and no cached response in nginx server. After `10` concurrent requests (to the same resource, the same cache key) completed, the result looks like :
  ```shell
  > ./h2load --requests=10 --clients=10  --verbose  https://localhost:8050/file?xxx_id=98760
  starting benchmark...
  spawning thread #0: 10 total client(s). 10 total requests
  Application protocol: h2
  [stream_id=1] :status: 200
  [stream_id=1] server: nginx/1.23.3
  [stream_id=1] date: Sat, 25 Feb 2023 14:54:47 GMT
  [stream_id=1] content-type: application/octet-stream
  [stream_id=1] cache-control: max-age=60
  [stream_id=1] x-cache-status: MISS
  progress: 10% done
  [stream_id=1] :status: 200
  [stream_id=1] server: nginx/1.23.3
  [stream_id=1] date: Sat, 25 Feb 2023 14:54:47 GMT
  [stream_id=1] content-type: application/octet-stream
  [stream_id=1] cache-control: max-age=60
  [stream_id=1] x-cache-status: HIT
  progress: 20% done
  ..... rest of 8 responses will be the same, cache hit .....
  finished in 1.62s, 6.17 req/s, 1.73MB/s
  requests: 10 total, 10 started, 10 done, 10 succeeded, 0 failed, 0 errored, 0 timeout
  status codes: 10 2xx, 0 3xx, 0 4xx, 0 5xx
  ```
  the 1st request completed with the header `x-cache-status: MISS`, which means it was passed to upstream server and the response was cached at here; while other 9 requests were blocking until the cached response was ready, then they responded with the cached data and the header `x-cache-status: HIT`
  
- Again with the example, except the cached response expired. After `10` concurrent requests completed, the result looks like :
  ```shell
  > ./h2load --requests=10 --clients=10  --verbose  https://localhost:8050/file?xxx_id=98760
  starting benchmark...
  spawning thread #0: 10 total client(s). 10 total requests
  Application protocol: h2
  [stream_id=1] :status: 200
  [stream_id=1] server: nginx/1.23.3
  [stream_id=1] date: Sat, 25 Feb 2023 14:56:57 GMT
  [stream_id=1] content-type: application/octet-stream
  [stream_id=1] cache-control: max-age=60
  [stream_id=1] x-cache-status: UPDATING
  progress: 10% done
  ..... rest of 8 responses will be the same, return stale cache data .....
  progress: 90% done
  [stream_id=1] :status: 200
  [stream_id=1] server: nginx/1.23.3
  [stream_id=1] date: Sat, 25 Feb 2023 14:56:58 GMT
  [stream_id=1] content-type: application/octet-stream
  [stream_id=1] cache-control: max-age=60
  [stream_id=1] x-cache-status: EXPIRED
  progress: 100% done
  finished in 723.49ms, 13.82 req/s, 3.87MB/s
  requests: 10 total, 10 started, 10 done, 10 succeeded, 0 failed, 0 errored, 0 timeout
  status codes: 10 2xx, 0 3xx, 0 4xx, 0 5xx
  ```
  The first 9 requests arrived when nginx was still updating the stale cached data, because `proxy_cache_use_stale` is specified with `updating` option, so they responded with the stale cached data and the header `x-cache-status: UPDATING`; the last request got new cached response and the header `x-cache-status: EXPIRE`.

###  Limit in Nginx
####  Limit on Connection
- [`limit_conn_zone`](http://nginx.org/en/docs/http/ngx_http_limit_conn_module.html#limit_conn_zone) declares  **shared space** and **key**
  - key : labels for separating different state data for different conditions. For example, separate by client IP address [`$binary_remote_addr`](http://nginx.org/en/docs/http/ngx_http_core_module.html#var_binary_remote_addr)
  - shared space : maintain state data which is used to limit connections **per defined key**
- For `limit_conn`, the *connection* in Nginx documentation could mean
  - TCP connection, client with one IP may establish multiple TCP connections
  - HTTP/2 request in a TCP connection, client may start multiple concurrent HTTP/2 requests in a single established TCP connection. (TODO, test this case)
- In [the example above](#configuration)
  - the server allows each IP to have at most 1 *connection* to the endpoint `/file`, denies others (if there's more) and respond with `429` in order NOT to exceed the limit.
  - To know more about the behavior, you can
    - Start a upstream server in debug mode, add breakpoint somewhere in middle of handling the endpoint. When you establish the first connection (e.g. using tools like `curl`) and it comes to nginx server, it will be temporarily blocked at the upstream server.
    - Now the first connection is stuck (at the upstream server), you establish 2nd. connection to nginx server, the 2nd connection will be denied  by the nginx due to the limit you specified (which is 1 in this example).
####  Limit on Request
- [`limit_req_zone`](http://nginx.org/en/docs/http/ngx_http_limit_req_module.html#limit_req_zone) declares  **shared space**, **key**, (similar as `limit_conn_zone`) and **rate**
  - rate, requests per second `r/s` or minute `r/m` at [layer 7](https://en.wikipedia.org/wiki/Application_layer)
- If `r/s` is greater than 1, [`limit_req`](http://nginx.org/en/docs/http/ngx_http_limit_req_module.html#limit_req) has to be specified with argument `burst`.
  - nginx worker seems to process only one request by default ([source](https://serverfault.com/questions/851750), TODO:verify), and respond with error for all other concurrent requests (even when it doesn't exceed the limit)
  - `burst` means number of excessive inbound requests to queue, so the worker can handle them at a later time
  - The current workaround is to specify `burst` to rate-limit value `r/s` minus 1
- In [the example above](#configuration)
  - the server allows each IP to have at most 3 *requests* to the endpoint `/status`, denies others (if there's more) and respond with `430` in order NOT to exceed the limit.
  - To meet rate-limiting requirement, `burst` is set to `2` within `limit_req`
  - To know more about the behavior, you can
    -  send 10 concurrent HTTP requests to nginx using [load testing tools](https://www.reddit.com/r/devops/comments/39w336/) (for example I use [`h2load`](https://nghttp2.org/documentation/h2load-howto.html)) see the number of requests which are done successfully or failed.
    -  following textual report is the result of `h2load` command after 10 concurrent requests are done.
       ```
       ./h2load --requests=10 --clients=10  --verbose  https://localhost:8050/status
       .... skip handshake detail ....
       progress: 100% done
       finished in 710.68ms, 14.07 req/s, 27.16KB/s
       requests: 10 total, 10 started, 10 done, 3 succeeded, 7 failed, 0 errored, 0 timeout
       status codes: 3 2xx, 0 3xx, 7 4xx, 0 5xx
       traffic: 19.30KB (19765) total, 466B (466) headers (space savings 45.37%), 18.25KB (18692) data
       ```
#### Limit network bandwidth
- [The example above](#configuration) specifies a limit `limit_rate` on `17k` bytes to transmit per second when downloading any JPG image.
- Assume you download a picture `/xyz.jpg` using HTTP test tool (e.g. `curl`,`h2load`), you will see the average bandwidth will be approximate `17k` (I use `h2load` at here) :
  ```
  ./h2load --requests=1 --clients=1  --verbose  https://localhost:8050/xyz.jpg
  .... skip handshake detail ....
  progress: 100% done
  finished in 9.07s, 0.11 req/s, 17.63KB/s
  requests: 1 total, 1 started, 1 done, 1 succeeded, 0 failed, 0 errored, 0 timeout
  status codes: 1 2xx, 0 3xx, 0 4xx, 0 5xx
  traffic: 159.93KB (163769) total, 113B (113) headers (space savings 38.25%), 159.52KB (163346) data
  ```
#### Access Control
- In [the example above](#configuration), to access the endpoint `/status` :
  - the server accepts reqeusts form the 2 IP addresses `123.4.56.78` and `127.0.0.1`, by directive [`allow`](http://nginx.org/en/docs/http/ngx_http_access_module.html#allow)
  - then rejects requests from all other IP addresses by derictive [`deny`](http://nginx.org/en/docs/http/ngx_http_access_module.html#deny)


### Load Balancing in Nginx
Note:
- the term **upstream server** and **origin server** are interchangable in this section
#### Layer 7 (HTTP)
- [`upstream`](https://nginx.org/en/docs/http/ngx_http_upstream_module.html#upstream) directive defines a group of origin servers the Nginx server will pass client requests to.
- In [the example above](#configuration) there are 2 origin servers `localhost:8010` and `localhost:8011` in the block
- the default algorithm for load balancing is [round-robin](https://docs.nginx.com/nginx/admin-guide/load-balancer/http-load-balancer/#choosing-a-load-balancing-method), it works with parameter `weight` in [`server`](https://nginx.org/en/docs/http/ngx_http_upstream_module.html#server) block
  - the server `localhost:8010` (say `orig1`) has weight set to `3`
  - `localhost:8011` (say `orig2`) has weight set to `5`
  - The time series of the 2 origin servers receiving request will be like :
    ```
     orig2, orig1, orig2, orig1, orig2, orig2, orig1, orig2
    ```
  - nginx tries to distribute evenly to the origin servers, with respect to the weight parameters
    - In every 8 requests, `orig2` receives `5` requests and `orig1` receives `3`.
- `max_fails` and `fail_timeout` usually work together as [passive health check](https://docs.nginx.com/nginx/admin-guide/load-balancer/http-health-check/#passive-health-checks) 
  - For example, the server `localhost:8010` has `max_fails = 3` and `fail_timeout = 7s`
  - Assume the server `localhost:8010` is down and failed to process `3` requests in `7` seconds
  - Nginx marks `localhost:8010` as *unavailable*. (say at the time `T0`), meanwhile it tries to pass the `3rd` request to differnt origin sevrer (might be `localhost:8011`).
  - From here, the Nginx temporarily passes new requests to other upstream servers except `localhost:8010`  for another `7` seconds, until the timeout occures (at the time `T0 + 7`), then Nginx retries sending a request to `localhost:8010`:
    -  if `localhost:8010` is restarted and able to handle request, then mark it as *active*
    -  otherwise Nginx repeats the same behavior, distributes requests to other upstream servers except `localhost:8010` until next timeout.
  - Nginx logs error message as soon as it failed to send a request to any origin server :
    ```
    2023/02/22 15:34:20 [error] 9814#9814: *7 connect() failed (111: Connection refused) while connecting to upstream, client: 127.0.0.1, server:
    localhost, request: "GET /file?xxx_id=987  HTTP/2.0", upstream: "https://127.0.0.1:8010/file?xxx_id=987", host: "localhost:8050"
    ```
  - Nginx logs warning message after it marked an origin server as  *unavailable* :
    ```
    2023/02/22 15:38:20 [warn] 9814#9814: *21 upstream server temporarily disabled while connecting to upstream, client: 127.0.0.1, server:
    localhost, request: "GET /file?xxx_id=987  HTTP/2.0", upstream: "https://127.0.0.1:8010/file?xxx_id=987", host: "localhost:8050"
    ```


#### Layer 4 (TCP/UDP)
(TODO)

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

### TODO
- Regex location might not be able to work with `proxy_pass` directive ?
- if `no-cache` is present in the header `cache-control`, does `proxy_no_cache` still take effect ?
- `ssl_session_cache`:
  - official doc describes the directive works with session ticket of TLS v1.3, however it is NOT clear whether the nginx has to be after version 1.23.2
  - how to measure running time and benchmark the performance gain from the directive ??
- Find out any method / directives for rate-limiting at [layer 4](https://en.wikipedia.org/wiki/Transport_layer)
- Try [Dynamic bandwidth control](https://docs.nginx.com/nginx/admin-guide/security-controls/controlling-access-proxied-http/#dynamic-bandwidth-control)
- try different load-balancing methods (except default round-robin)
- try `resolver` directive for DNS lookup
- try [active health check](https://docs.nginx.com/nginx/admin-guide/load-balancer/http-health-check/#active-health-checks), which is not in open source version

### Reference
* [Build Nginx from source -- MatthewVance](https://github.com/MatthewVance/nginx-build/blob/master/build-nginx.sh)
* [How To Compile Nginx From Source and Install on Raspbian Jessie](https://www.linuxbabe.com/raspberry-pi/compile-nginx-source-raspbian-jessie)
* [Build Nginx from source - offiical doc](https://docs.nginx.com/nginx/admin-guide/installing-nginx/installing-nginx-open-source/)
* [Configure Nginx running with uWSGI](https://uwsgi-docs.readthedocs.io/en/latest/tutorials/Django_and_nginx.html#basic-nginx)
* [How To Set Up uWSGI and Nginx to Serve Python Apps on Ubuntu 14.04](https://www.digitalocean.com/community/tutorials/how-to-set-up-uwsgi-and-nginx-to-serve-python-apps-on-ubuntu-14-04)
* [Nginx full-example configuration](https://www.nginx.com/resources/wiki/start/topics/examples/full/)
* [Avoiding the Top 10 NGINX Configuration Mistakes](https://www.nginx.com/blog/avoiding-top-10-nginx-configuration-mistakes/)
* [A Guide to Caching with NGINX and NGINX Plus](https://www.nginx.com/blog/nginx-caching-guide/)
* [Nginx HTTP load balancing](https://docs.nginx.com/nginx/admin-guide/load-balancer/http-load-balancer/)
* [ServerFault - NGINX proxy cache time with Cache-Control](https://serverfault.com/questions/915463)
* [curl issue -- openssl: support session resume with TLS 1.3](https://github.com/curl/curl/pull/3271)
* [curl mail list thread -- TLS session ID re-use broken in 7.77.0](https://curl.se/mail/lib-2021-06/0016.html)
* [ServerFault -- Nginx `limit_req_zone` limiting at a rate lower than specified](https://serverfault.com/questions/851750)
* [Nginx config generator -- Digital Ocean](https://www.digitalocean.com/community/tools/nginx)
* [Nginx Cookbook, chapter 4 - Massively Scalable Content Caching (trial)](https://www.oreilly.com/library/view/nginx-cookbook/9781492078470/ch04.html)
