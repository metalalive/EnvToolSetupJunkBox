### Accesses through REST API

perform all operations through REST API, basic pattern is like :
```
curl  -H "Content-Type: YOUR_CONTENT_TYPE;"  -H "Accept: YOUR_ACCEPT_TYPE;"   --request REQEUST_METHOD \
  --data-binary '@request_body_file'  "http://HOSTNAME:PORT/API_PATH"
```

Note:
* `YOUR_CONTENT_TYPE`, `YOUR_ACCEPT_TYPE` is mostly `application/json` , there would be other available mime-types
* `--data-binary '@request_body_file' ` is optional , depending on design of each API endpoint.
* `API_PATH` varies between each API endpoint


### Status check

#### [cat health ](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/cat-health.html)
```
curl -H "Accept: application/json;"   --request GET "http://HOSTNAME:PORT/_cat/health?pretty"
```
which returns list of running clusters :
```json
// here only few important fields are shown here to check
[
  {
    // .....
    "cluster" : "my-application",
    "status" : "yellow",
    "node.total" : "1",
    "node.data" : "1",
    "shards" : "15",
    "pri" : "15",
    "relo" : "0",
    // .......
    "active_shards_percent" : "50.0%"
  }
]
```

Note
* you can add `pretty` to URI parameter section, to ask for more presentable response data from elasticsearch server.
*  `node.total` depends on number of running nodes (elasticsearch processes)
* `status` could be `green`, `yellow`, `red`,  read [this](https://www.elastic.co/guide/en/elasticsearch/reference/current/cluster-health.html#cluster-health-api-response-body) for detail.


#### [Nodes](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/cat-nodes.html)
```
curl  -H "Accept: application/json;"   --request GET "http://HOSTNAME:PORT/_cat/nodes?v&pretty"
```
which returns list of running nodes like this:
```json
[
  {
    "ip" : "YOUR_HOSTNAME",
    "heap.percent" : "21",
    "ram.percent" : "93",
    "cpu" : "5",
    "load_1m" : "0.11",
    "load_5m" : "0.26",
    "load_15m" : "0.31",
    "node.role" : "mdi",
    "master" : "*",
    "name" : "YOUR_NODE_NAME"
  }
]
```

### Index
* List all existing indices
```
curl  -H "Accept: application/json;"   --request GET  "http://HOSTNAME/_cat/indices?v&pretty"
```

* Create a new index
```
curl  -H "Accept: application/json;"  --request PUT "http://HOSTNAME:PORT/NEW_INDEX_NAME?pretty"
```
Remember index name (`NEW_INDEX_NAME`) has to be **all lower-case characters**, underscore character is allowed.
Response would be like :
```
// acknowledged is true, which means new index is successfully created.
{
  "acknowledged" : true,
  "shards_acknowledged" : true,
  "index" : "NEW_INDEX_NAME"
}
```

* Delete an existing index
```
curl  -H "Accept: application/json;"  --request DELETE "http://HOSTNAME:PORT/NEW_INDEX_NAME?pretty"
```
Response would be like :
```
{
  "acknowledged" : true
}
```

### Document

Documents in Elasticsearch is like rows/records of a table in SQL database world

#### Add a new document

command
```shell
curl  -H "Content-Type: application/x-ndjson"  -H "Accept: application/json"   --data-binary  '@REQ_BODY_FILE' \
    --request POST "http://HOSTNAME:PORT/CHOSEN_INDEX/CHOSEN_TYPE/USER_DEFINED_ID?pretty" | less
```
request body file (`REQ_BODY_FILE`) may be like:
```json
{"name": "Rei", "unit": ["millisecond", "gallon"]}
```
response:
```
{
  "_index" : "CHOSEN_INDEX",
  "_type" : "CHOSEN_TYPE",
  "_id" : "USER_DEFINED_ID",
  "_version" : 1,
  "result" : "created",
  "_shards" : {
    "total" : 2,
    "successful" : 1,
    "failed" : 0
  },
  "created" : true
}
```
Note
* header should include `Content-Type` with value `application/x-ndjson` (not `application/json`), since I use  the option `--data-binary` in the command, which means data source of request body is from binary file, not from command-line string.
* there must be `@` character before `REQ_BODY_FILE` in the command
* `CHOSEN_INDEX` could be any existing index , the same rule is also applied to `CHOSEN_TYPE`
* `USER_DEFINED_ID` is any string that hasn't been used as ID for any other document **under the same index and the same type**.
* You have to choose an index when creating a new document



#### Update an existing document
##### Update and erase all previous stored fields

If you run the same command as shown in [Add a new document](#add-a-new-document) again, with the same index / type / ID but different request body data, elasticsearch will internally treat it as an update operation and **discard all the fields stored in the document in your previous API call**.

response:
```
{
  "_index" : "CHOSEN_INDEX",
  "_type" : "CHOSEN_TYPE",
  "_id" : "USER_DEFINED_ID",
  "_version" : 2,
  "result" : "updated",
  "_shards" : {
    "total" : 2,
    "successful" : 1,
    "failed" : 0
  },
  "created" : false
}
```

There are few idfferences if you compare this response with the previous one :
| field | this response | previous response |
|-----------|------------|-----------|
| `result` |  `updated` | `created` |
| `created` |  `false` | `true` |


##### Update with partial document

You can also select which fields to update (and unselected fields remain unchanged) in a document, by calling `/CHOSEN_INDEX/CHOSEN_TYPE/USER_DEFINED_ID/_update`  API endpoint

```
curl  -H "Content-Type: application/json"  -H "Accept: application/json;"  --data-binary  '@REQ_BODY_FILE' \
    --request POST "http://HOSTNAME:PORT/CHOSEN_INDEX/CHOSEN_TYPE/USER_DEFINED_ID/_update?pretty"
```

Assume a document already includes following fields
```json
{
    "name": "Rin",
    "last_comment": "awesome work",
    "num_likes": 345
}
```
And you attempt to add/update some fields in one shot, so the file `REQ_BODY_FILE` contains following data :
```json
{"doc": {"donate": {"amount": 50, "currency": "RMB_cent"}, "last_comment": "love it"}}
```
After API call to `/CHOSEN_INDEX/CHOSEN_TYPE/USER_DEFINED_ID/_update` , the document will be :
```json
{
    "name": "Rin",
    "last_comment": "love it",
    "donate": {"amount": 50, "currency": "RMB_cent"},
    "num_likes": 345
}
```

Note :
* `doc` field in the request body file (as shown above) specify new fields to add , or existing fields to update
* In update cases, the pattern of request body should look like `{ DOC_OR_SCRIPT: DETAIL_CONTENT }`
  * key field `DOC_OR_SCRIPT` can be either of [doc](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/docs-update.html#_updates_with_a_partial_document) or [script](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/docs-update.html#_scripted_updates)
  * value field `DETAIL_CONTENT` depends on its key field
  * both `doc` and `script` [cannot be placed in the same clause](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/docs-update.html#_updates_with_a_partial_document), e.g. `{ "doc": DOC_DETAIL, "script": SCRIPT_DETAIL }` is invalid syntax in elasticsearch.
* To delete fields during update API call, you need to write script `{"script" : {"source": "ctx._source.remove('your_field_name')" }}`
  * the `ctx` is a map for a document to access its metadata or field data.
  * `ctx` has several attributes to access: `_source`, `_index`, `_type`, `_id`, `_version`, `_routing`, `_parent` ... etc

