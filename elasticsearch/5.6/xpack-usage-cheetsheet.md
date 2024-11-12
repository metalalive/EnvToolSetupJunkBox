## X-pack usage cheetsheet

> You may need to add a few options ONLY if :
> - x-pack plugin is installed
> - and the license is valid (not expired)

### Authentication
After x-pack installation, Authentication is **required** by default for most (all?) of API calls, be sure to add valid username & password in each API request, Otherwise you would receive `401` error response.
```
curl  --header "Content-Type: application/json"  --header "Accept: application/json;"  --request <WHATEVER_METHOD> \
    "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/<WHATEVER_API_ENDPOINT>" 
```

### Account Management

X-pack plugin is required for following API endpoints, also make sure user has the privilege to perform these operations

#### Create User
```
curl -s --header "Content-Type: application/x-ndjson" --header "Accept: application/json" --data-binary '@es_xpack_edit_user.json' \
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

#### Edit User

API endpoint is the same as above `/_xpack/security/user/<EXISTING_USER_NAME>`, but request method is `PUT`, also note that :
* `password` field can be omitted in the request body
* It is NOT partial update, the fields specified in the update API call will overwrite whole content of the fields stored in elasticsearch accordingly. For example, if you attempt to only append new roles to the list of `roles` field , you will need to fetch those roles already assigned to the user.
* Built-in users in elasticsearch can NOT be updated by anyone, otherwise you will get error response (validation failure, HTTP status 400)

#### View status of user(s)
* For any authenticated user viewing him/herself:
```
curl -s --header "Accept: application/json" --request GET \
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
curl -s --header "Accept: application/json" --request GET \
    "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/user/?pretty"
```
Then elasticsearch responds with list of exising users, the structure of each item is as shown above.

#### Change password
```
curl  --header "Content-Type: application/json"  --header "Accept: application/json;" -d '{"password": "<YOUR_NEW_PASSWD>"}' \
   --request PUT  "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/user/<USERNAME>/_password?pretty" 
```
Note:
* Each user account can only change his/her own password, unless `USERNAME` has superuser role.


### Role Management
#### Create a role
```
curl -s --header "Content-Type: application/x-ndjson" --header "Accept: application/json" --data-binary '@es_xpack_edit_role.json' \
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

#### Update a role
API endpoint is the same as above `/_xpack/security/role/<EXISTING_ROLE_NAME>`, but request method is `PUT`, also note that :
* `password` field can be omitted in the request body
* It is NOT partial update, the fields specified in the update API call will overwrite whole content of the fields stored in elasticsearch accordingly.
* Built-in roles in elasticsearch can NOT be updated by anyone, otherwise you will get error response (validation failure, HTTP status 400)

#### View status of role(s)
For users who have permission to view all existing roles :
```
curl -s --header "Accept: application/json" --request GET \
    "http://<USERNAME>:<PASSWORD>@<HOSTNAME>:<PORT>/_xpack/security/role/?pretty"
```
