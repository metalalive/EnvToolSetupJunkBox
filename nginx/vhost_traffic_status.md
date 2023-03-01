## Concept
### Memory zone
- specified by `vhost_traffic_status_zone`, shared among all worker processes
- keeps collected data for different [user-defined conditions](#filter)
- automatically adjusted its size when more keys are added.
### Filter
- user-defined condition (in a http/server/location block), specified using directives like `vhost_traffic_status_filter_by_set_key`
- split entire collected data to different subsets, based on the condition
- dump necessary information under the specific condition
- For example, the condition could be :
  - status code of HTTP response, `$status` e.g. to group the traffic data to several subsets, each of them (e.g. `200`, `201`, `404`), e.g. your application server returns codes 
  - URI, `$uri`, in case you're monitoring traffic volume through a few API endpoints
  - query string, `$query_string`, in case you're interested with some query parameters of a specific API endpoint 
### Server
- by default, [`nginx-module-vts`](https://github.com/vozlt/nginx-module-vts) is able to dump necessary traffic data for specific server blocks (the `server_name` directives)
### Group
- or group label (TODO)

## Configuration
See [this](./core_setup.md#configuration) for detail. In the rest of this study note I will explain how this module works using the configuration example.

## Directives
One filter is defined in the location block `/file`:
```
vhost_traffic_status_filter_by_set_key   $uri   $upstream_addr::non-stream-file::*;
```
### `vhost_traffic_status_filter_by_set_key`
- the `key` argument is **the condition for partitions** (on enture status data), which can be either *nginx variable* or *constant string*
- the `name` argument is a **group label** , which can be mix of *nginx variable* and *constant string*. e.g. `$upstream_addr::non-stream-file::*`
- note that `$uri` represents the path without **query parameter string**
  - e.g. URI is `/mycard/yesno?xxxid=98765`, the `$uri` after nginx relosion will be `/mycard/yesno`
- to fetch query parameter string as a variable, use `$query_string` instead
  - downside: this will create more keys maintained in a [memory zone](#memory-zone), use it carefully.

### `vhost_traffic_status_filter_max_node`
- limit number of nodes (different **group labels**) created for different [filtering conditions](#vhost_traffic_status_filter_by_set_key)
- for example in the configuration, the upstream `backend_app_345` includes 2 origin servers
  - the filter will have 2 group labels
  - max number of group labels is `17`

### `vhost_traffic_status_display`
`vhost_traffic_status_display` specified in the location block `/status`:
```
location /status {
    vhost_traffic_status_display;
    vhost_traffic_status_display_format  html;
    .... skip unimportant part ....
}
```
which makes the location `/status` be an access point of collected traffic data, users can perform simple operations on it.
- read
  - data in all **filter zones** and all **server zones**, in [different formats](https://github.com/vozlt/nginx-module-vts#description)
  - data only in all **filter zones** : `/status/control?cmd=status&group=filter&zone=*`
  - data only in all **server zones** : `/status/control?cmd=status&group=server&zone=*`
- reset all **filter zones**
  - `/status/control?cmd=reset&group=filter&zone=*`
- reset **server zones** `localhost`
  - `/status/control?cmd=server&group=server&zone=localhost`
- currently the access to a single filter zone is NOT supported. (see [TODO](#TODO))
  - this does not work `/status/control?cmd=reset&group=filter&zone=`


## Useful metrics
- The metrics in this section are present in json format, as normal response body of the request `/status/control?cmd=status&group=filter&zone=*`.
- All filter groups are initialized to zero on nginx start, and discarded (not persisted) after nginx shutdown
- All filter groups reflect specific time point, NOT a period, to draw a time-series chart you'll need to save the metrics somewhere (e.g. [TSDB](https://en.wikipedia.org/wiki/Time_series_database))
### Usage scenario
Assume an client attempts to request for a non-existing file in the origin server, it should not be cached in nginx proxy server (so passing the request to upstream server), and the nginx will respond with `404` status code.
```
curl  --write-out @/path/to/performance.log -v  "https://localhost:8050/file?xxx_id=non-exist-res-identity"
```
assume the performance log file will be:
```
 time_namelookup:  0.000767s
    time_connect:  0.000875s
 time_appconnect:  0.007406s
time_pretransfer:  0.008007s
   time_redirect:  0.000000s
time_starttransfer:  0.092607s
 ----------
      time_total:  0.092893s
```
Then check out the traffic data
```
curl  "https://localhost:8050/status/control?cmd=status&group=filter&zone=*"
```
the response body will be look :
```json
{
    "filterZones": {
        "127.0.0.1:8010::non-stream-file::*": {
            "/file": { "requestCounter": 1,
                "responses": {
                    "1xx": 0,  "2xx": 0,  "3xx": 0,  "4xx": 1,  "5xx": 0,
                    "miss": 1,  "bypass": 0,  "expired": 0,  "stale": 0,
                    "updating": 0,  "revalidated": 0,  "hit": 0,  "scarce": 0
                },  "requestMsecCounter": 84, "requestMsec": 84,
                .....
            },
        }, // end of the filter group
        .....
    } // end of filterZones
}
```
#### Request counter
- `requestCounter` should be equal to `1xx + 2xx + 3xx + 4xx + 5xx`
#### Responses in each filter group
- In the `responses` object, all fields are accumulative counters
  - `1xx`, `2xx`, ......, `5xx` categorize responses by its status code
  - rest of fields e.g. `miss`, `hit`, `expired` means result of caching operation in nginx proxy server
- One filter group `127.0.0.1:8010::non-stream-file::*` was created
  - the `4xx` field is `1`, which means the client got `404` response status when requesting non-existing file
  - the `miss` field is `1`, means one cache miss the due to the `404` response above
#### Processing time in millisecond
- `requestMsec` indicates the total time of **the latest finished request**, in the example it is `84` ms, slightly lower than `92.8` ms (recorded by `curl`)
- `requestMsecCounter` is an accumulative counter for all the past `requestMsec` values

If the client keeps sending the same request (say 6 times, 7 in total), the collected traffic data will look like :
```json
{
    "filterZones": {
        "127.0.0.1:8010::non-stream-file::*": {
            "/file": { "requestCounter": 4,
                "responses": {
                    "1xx": 0,  "2xx": 0,  "3xx": 0,  "4xx": 4,  "5xx": 0,
                    "miss": 4,  "bypass": 0,  "expired": 0,  "stale": 0,
                    "updating": 0,  "revalidated": 0,  "hit": 0,  "scarce": 0
                },  "requestMsecCounter": 268, "requestMsec": 41,
                .....
            },
        }, // end of the filter group
        "127.0.0.1:8011::non-stream-file::*": {
            "/file": { "requestCounter": 3,
                "responses": {
                    "1xx": 0,  "2xx": 0,  "3xx": 0,  "4xx": 3,  "5xx": 0,
                    "miss": 3,  "bypass": 0,  "expired": 0,  "stale": 0,
                    "updating": 0,  "revalidated": 0,  "hit": 0,  "scarce": 0
                },  "requestMsecCounter": 94, "requestMsec": 31,
                .....
            },
        }, // end of the filter group
        .....
    } // end of filterZones
}
```
According to the configuration example
- Round-robin algorithm is applied to the upstream block
- 2 origin servers `127.0.0.1:8010` and `127.0.0.1:8010` in the upstream block `backend_app_345`
- 2 corresponding filter groups are present, the nginx tried to evenly distributed all the requests to the 2 active origin sevrers.
  - In total 7 requests, 3 of them passed to `127.0.0.1:8011`, 4 of them passed to `127.0.0.1:8010`

|filter group|`127.0.0.1:8010::non-stream-file::*`|`127.0.0.1:8011::non-stream-file::*`|
|------------|----------|-----------|
|`4xx`| 4| 3|
|`miss`| 4| 3|
|`requestMsec`| 268| 94|
|`requestMsecCounter`| 41| 31|


## TODO
- Figure out how to use the directive `vhost_traffic_status_histogram_buckets` and what **histogram** means in official document
- figure out `filterZones` and `serverZones` in document, the description is unclear 
- Follow [this issue](https://github.com/vozlt/nginx-module-vts/issues/135), unfinished enchancement of filter-level group control.

## Reference
- [Nginx virtual host traffic status module - Github](https://github.com/vozlt/nginx-module-vts)
