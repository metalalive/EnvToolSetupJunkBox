### Concept
#### Memory zone
- specified by `vhost_traffic_status_zone`, shared among all worker processes
- keeps collected data for different [user-defined conditions](#filter)
- automatically adjusted its size when more keys are added.
#### Filter
- user-defined condition (in a http/server/location block), specified using directives like `vhost_traffic_status_filter_by_set_key`
- split entire collected data to different subsets, based on the condition
- dump necessary information under the specific condition
- For example, the condition could be :
  - status code of HTTP response, `$status`, e.g. your application server returns codes like 200, 201 or , 403
  - URI, `$uri`, in case you're monitoring traffic volume through a few API endpoints
  - query string, `$query_string`, in case you're interested with some query parameters of a specific API endpoint 
#### Server
- by default, [`nginx-module-vts`](https://github.com/vozlt/nginx-module-vts) is able to dump necessary traffic data for specific server blocks (the `server_name` directives)
#### Group
- or group label (TODO)

### Configuration example
See the [configuration section](./core_setup.md#configuration) in the study note.
#### `vhost_traffic_status_filter_by_set_key`
- the `key` argument is **the condition for partitions on enture status data** , which can be either *nginx variable* or *constant string*
- the `name` argument is a **group label** , which can be mix of *nginx variable* and *constant string*. e.g. `$upstream_addr::non-stream-file::*`
- note that `$uri` represents the path without **query parameter string**
  - e.g. URI is `/mycard/yesno?xxxid=98765`, the `$uri` after nginx relosion will be `/mycard/yesno`
- to fetch query parameter string as a variable, use `$query_string` instead
  - downside: this will create more keys to maintain in the [memory zone](#memory-zone), use it carefully.
#### `vhost_traffic_status_filter_max_node`
- limit number of nodes (different **group labels**) created for different [filtering conditions](#vhost_traffic_status_filter_by_set_key)
- for example in the configuration, the upstream `backend_app_345` includes 2 origin servers
  - the filter will have 2 group labels
  - max number of group labels is `17`

### Control

`/status/control?cmd=reset&group=server&zone=localhost`

`/status/control?cmd=reset&group=filter&zone=*`

does not work
`/status/control?cmd=reset&group=filter&zone=`


### TODO
- Figure out how to use the directive `vhost_traffic_status_histogram_buckets` and what **histogram** means in official document
- figure out `filterZones` and `serverZones` in document, the description is unclear 
- Follow [this issue](https://github.com/vozlt/nginx-module-vts/issues/135), unfinished enchancement of filter group control.

### Reference
- [Nginx virtual host traffic status module - Github](https://github.com/vozlt/nginx-module-vts)
