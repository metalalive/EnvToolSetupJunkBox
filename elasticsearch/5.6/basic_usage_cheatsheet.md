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

#### [License](https://www.elastic.co/guide/en/elasticsearch/reference/current/license-settings.html)
By default, the license type is `trial` on new fresh installation, after `version >= 6.3` , elasticsearch has provided open source version of elasticsearch to install (OSS in short) since version `6.3`. The [license type](https://www.elastic.co/subscriptions) affects the functionalities that are allowed to use in elasticsearch e.g. Xpack, machine-learning.

* check your license type, and expiry time :
```
curl -s -H "Accept: application/json" --request GET "http://USERNAME:PASSWORD@HOSTNAME:PORT/_license?pretty"
```

unfortunately, most functions of x-pack plugin are DISABLED under **basic license** privilege.

For older versions (`version < 6.3`), you can get a **basic license** for free by requesting a key file that will be used to register and get the **basic license** (will be expired after one year). Here are the steps :
* start with filling the request form at [here](https://license.elastic.co/registration/). 
* On the form submission, the website will email you with subsequent steps you must follow, and dowdnload link to a key file
* download the key file (e.g. `DOWNLOADED_KEY_FILE_PATH`), update your elasticsearch license by consuming the API :
```
curl -s -H "Content-Type: application/x-ndjson" -H "Accept: application/json" --data-binary \
    '@DOWNLOADED_KEY_FILE_PATH' --request PUT  "http://USERNAME:PASSWORD@HOSTNAME:PORT/_xpack/license?pretty"
```
Note:
Without URL query parameter `acknowledge=true`, the key file won't be used and **basic license** won't be applied, instead you will get message to list all functions that will be disabled after applying the basic license, after reading through all effects , you can add the query parameter `acknowledge=true` back to the API URL, this time it will truly apply the key file and update to **basic license** for you.

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
curl  -H "Accept: application/json;"  --request DELETE "http://HOSTNAME:PORT/USELESS_INDEX_NAME?pretty"
```
Note:
* `USELESS_INDEX_NAME` may contain wildcard character, e.g. `serverlog-2020-*`

Response would be like :
```
{
  "acknowledged" : true
}
```
### Mapping

By default, Elasticsearch guesses data type on each field  when inserting a [document](#document) with a new index that hasn't been created. Default data types somehow may bring up issues like [Array of objects (nested fields) is flattened](https://www.elastic.co/guide/en/elasticsearch/reference/current/nested.html#nested-arrays-flattening-objects) , once data type of a field is determined for an index (for a set of indices), [there is no easy way to edit that after your first indexed document is stored](https://discuss.elastic.co/t/how-to-update-a-field-type-of-existing-index-in-elasticsearch/53892).
* 

If you want to specify non-default data type to any field of your document, you can customize [mapping](https://www.elastic.co/guide/en/elasticsearch/reference/current/mapping.html#mapping) definition before creating your first document with a new index. See [this article](https://www.elastic.co/blog/logstash_lesson_elasticsearch_mapping) to create custom template for index mapping.

#### Check your template(s)

```
curl -s -H "Accept: application/json" --request GET "http://HOSTNAME:PORT/_template?pretty"

curl -s -H "Accept: application/json" --request GET "http://HOSTNAME:PORT/_template/YOUR_TEMPLATE_NAME?pretty"
```

#### Create / Update a template
```
curl -s -H "Content-Type: application/x-ndjson" -H "Accept: application/json" --data-binary '@index_mapping_template.json' \
    --request PUT "http://HOSTNAME:PORT/_template/YOUR_TEMPLATE_NAME?pretty"
```

The tmeplate example [index_mapping_template.json](./index_mapping_template.json) could be build by copying everything read from the API `GET /_template/YOUR_TEMPLATE_NAME` , with some modification, then place them to HTTP request body . Elasticsearch will acknowledge the request once succeed :
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


#### Delete an existing document

By simply send `DELETE` request with the document ID you attempt to delete, see [reference](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/_deleting_documents.html)

```
curl  -H "Content-Type: application/json"  -H "Accept: application/json;"  --data-binary  '@REQ_BODY_FILE' \
    --request DELETE "http://HOSTNAME:PORT/CHOSEN_INDEX/CHOSEN_TYPE/USER_DEFINED_ID?pretty"
```
or you can selectively delete several documents by providing query :
```
curl -s -H "Content-Type: application/x-ndjson" -H "Accept: application/json" --data-binary '@es_search_query.json'  --request POST "http://<HOSTNAME>:<PORT>/<CHOSEN_INDEX>/<CHOSEN_TYPE>/_delete_by_query?pretty"
```
where `es_search_query.json` includes search conditions, for example, to delete all documents under an index, you have:
```json
{
    "query": {
        "match_all": {}
    }
}
```

### Batch processing

```
curl -s -H "Content-Type: application/x-ndjson"  -H "Accept: application/json;"  --data-binary  '@REQ_BODY_FILE' \
    --request POST  "http://HOSTNAME:PORT/_bulk?pretty"
```
According to [the general structure of the endpoint `_bulk`](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/docs-bulk.html#docs-bulk) , the file that contains request body data (`REQ_BODY_FILE`) may look like:
```json
// Here are all possible payloads in one API call, the actual payload data of request body depends on your use cases.
{"index": {"_index": CHOSEN_INDEX, "_type": CHOSEN_TYPE}}
{"field_name_1": 1234, "field_name_2": "some_value"}
{"create": {"_index": CHOSEN_INDEX, "_type": CHOSEN_TYPE, "_id": USER_DEFINED_ID}}
{"field_name_3": 456, "field_name_4": "some_value"}
{"update": {"_index": CHOSEN_INDEX, "_type": CHOSEN_TYPE, "_id": USER_DEFINED_ID}}
{"doc": {"field_name_5": "some_value", "field_name_6": 78}}
{"update": {"_index": CHOSEN_INDEX, "_type": CHOSEN_TYPE, "_id": USER_DEFINED_ID}}
{"script": {"source": "ctx._source.remove(params.del_field_name)", "lang":"painless", "params":{"del_field_name": "field_name_7"}}}
{"delete": {"_index": CHOSEN_INDEX, "_type": CHOSEN_TYPE, "_id": USER_DEFINED_ID}}

```

Note:
* The header `content-type` should be `application/x-ndjson` , not `application/json`
* If index (and type) is provided in URI (e.g. `/CHOSEN_INDEX/_bulk` or `/CHOSEN_INDEX/CHOSEN_TYPE/_bulk`), then it is NOT necessary to explicitly provide the values in `_index` field (and `_type` field)  of each payload of request body.
* The command part for each payload can be `index`, `create`, `update`, `delete`.
* Both of `index` and `create` are used to add a new document, the (only ?) difference seems to be : `index` command could automatically generate ID value for a new document if you don't provide `_id` field, while `create` command requires `_id` field specified in the payload of request body. 
* If you need to add, update, and delete fields of a document in one shot , you have to separate `doc` field (for adding and updating fields) and `script` field (for deleting fields) into 2 payloads, DO NOT put them into one payload otherwise [you will get validation failure](https://stackoverflow.com/a/65147914/9853105)
* official documentation describes that the request body should end with newline character (ASCII: `0xA`) for bulk API, [But in some versions, the bulk API call seems to work even you don't do so](https://stackoverflow.com/questions/36766471/validation-failed-1-no-requests-added-in-bulk-indexing-elasticsearch#comment115145269_36769643) .... Keep that in mind.




### X-pack
You may need to add a few options ONLY if x-pack plugin is installed

#### Authentication
After x-pack installation, Authentication is **required** by default for most (all?) of API calls, be sure to add valid username & password in each API request, Otherwise you would receive `401` error response.
```
curl  -H "Content-Type: application/json"  -H "Accept: application/json;"  --request <WHATEVER_METHOD> \
    "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/<WHATEVER_API_ENDPOINT>" 
```


#### Account Management

X-pack plugin is required for following API endpoints, also make sure user has the privilege to perform these operations

##### Create User
```
curl -s -H "Content-Type: application/x-ndjson" -H "Accept: application/json" --data-binary '@es_xpack_edit_user.json' \
    --request POST  "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/user/<NEW_USER_NAME>?pretty"
```
where `es_xpack_edit_user.json` may be like:
```json
{
    "password": "<INITIAL_PASSWORD>",
    "roles" : ["<ASSIGNED_ROLE_1>",  "<ASSIGNED_ROLE_2>", "<ASSIGNED_ROLE_3>" ],
    "full_name" : "<WHATEVER_NAME>",
    "email" : null,
    "enabled": true
}
```
Note:
* `<ASSIGNED_ROLE_x>` is valid name of an existing role in your elasticsearch, see [how to view/edit a role](#role-management) for detail.

##### Edit User

API endpoint is the same as above `/_xpack/security/user/<EXISTING_USER_NAME>`, but request method is `PUT`, also note that :
* `password` field can be omitted in the request body
* It is NOT partial update, the fields specified in the update API call will overwrite whole content of the fields stored in elasticsearch accordingly. For example, if you attempt to only append new roles to the list of `roles` field , you will need to fetch those roles already assigned to the user.
* Built-in users in elasticsearch can NOT be updated by anyone, otherwise you will get error response (validation failure, HTTP status 400)

##### View status of user(s)
* For any authenticated user viewing him/herself:
```
curl -s -H "Accept: application/json" --request GET \
    "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/_authenticate?pretty"
```
expected response may look like:
```json
{
  "username" : "<USERNAME>",
  "roles" : ["<ASSIGNED_ROLE_1>",  "<ASSIGNED_ROLE_2>", "<ASSIGNED_ROLE_3>" ],
  "full_name" : "<WHATEVER_NAME>",
  "email" : null,
  "metadata" : { },
  "enabled" : true
}
```
Note the enabled field can be false, which means the user account is deactivated.

* For users who have permission to view all other users :
```
curl -s -H "Accept: application/json" --request GET \
    "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/user/?pretty"
```
Then elasticsearch responds with list of exising users, the structure of each item is as shown above.

##### Change password
```
curl  -H "Content-Type: application/json"  -H "Accept: application/json;" -d '{"password": "<YOUR_NEW_PASSWD>"}' \
   --request PUT  "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/user/<USERNAME>/_password?pretty" 
```
Note:
* Each user account can only change his/her own password, unless `USERNAME` has superuser role.


#### Role Management

##### Create a role
```
curl -s -H "Content-Type: application/x-ndjson" -H "Accept: application/json" --data-binary '@es_xpack_edit_role.json' \
    --request POST  "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/role/<NEW_ROLE_NAME>?pretty"
```
where `es_xpack_edit_role.json` may look like :
```
{
    "cluster":["<VALID_CLUSTER_PRIV_1>", "<VALID_CLUSTER_PRIV_2>"],
    "indices":[
        {
            "names": ["<INDEX_PATTERN_1>", "<INDEX_PATTERN_2>"],
            "privileges": ["<VALID_INDICES_PRIV_1>", "<VALID_INDICES_PRIV_2>"]
        }
    ],
    "run_as": []
}
```
Note:
* `<VALID_CLUSTER_PRIV_x>` is valid name of any low-level [cluster privilege](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/security-privileges.html#privileges-list-cluster) defined in elasticsearch, these privileges will take effect in the entire cluster.
* `<VALID_INDICES_PRIV_x>` is valid name of any low-level [indices privilege](https://www.elastic.co/guide/en/elasticsearch/reference/6.3/security-privileges.html#privileges-list-indices) defined in elasticsearch, these privileges will affect access permissions to the index patterns in the list : `<INDEX_PATTERN_1>`, `<INDEX_PATTERN_2>` .....
* The list of the valid privileges (as mentioned above) may change between different elasticsearch versions, unfortunately, the privileges are probably NOT documented for old versions (before v6.3), You might need trial and error ....
* `<INDEX_PATTERN_x>` may contain wildcard character `*` to cover variation of index string patterns, e.g. `log-*-appserver` 

##### Update a role
API endpoint is the same as above `/_xpack/security/role/<EXISTING_ROLE_NAME>`, but request method is `PUT`, also note that :
* `password` field can be omitted in the request body
* It is NOT partial update, the fields specified in the update API call will overwrite whole content of the fields stored in elasticsearch accordingly.
* Built-in roles in elasticsearch can NOT be updated by anyone, otherwise you will get error response (validation failure, HTTP status 400)

##### View status of role(s)
For users who have permission to view all existing roles :
```
curl -s -H "Accept: application/json" --request GET \
    "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/role/?pretty"
```


### Query, DSL
too complex ... might require another markdown file to describe

### Troubleshooting
#### out-of-memory error (JVM)
##### Symptom
health flag turns RED, a running node goes down.
##### Possible rootcuase
certain number of documents indexed in elasticsearch, each of which has different key set, that cuase heap usage of index mapping grows.
##### Solution
For key-value pair in a document, [avoid key generation depending on different use cases](https://www.elastic.co/blog/found-crash-elasticsearch#mapping-explosion), make document structure consistent.



