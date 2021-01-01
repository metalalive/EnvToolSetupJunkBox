OS environment : Ubnutu / Debian system

### Run
#### run it as a service

service seems to launch only one node

```
service elasticsearch start
service elasticsearch stop
```

#### run it as a stand-alone process

which can launch several nodes

```
ES_JAVA_OPTS="<WHATEVER_IN_JVM_OPTIONS_FILE>" /ES_HOME/bin/elasticsearch -Epath.conf=/PATH/TO/CONFIG/FOLDER
```

Note
* `path.conf` exists only in old versions (probably before v6.0), removed in later version
* `/PATH/TO/CONFIG/FOLDER` contains few essential config files `jvm.options`, `log4j2.properties`, `elasticsearch.yml`.
* The default `ES_HOME` could be `/usr/share/elasticsearch` in Ubuntu/Linux system. 
* stand-alone elasticsearch process requires JVM arguments set in environment variable `ES_JAVA_OPTS`.
  * elasticsearch is likely unable to find out [jvm.options](#jvmoptions) if it is NOT located in default path `/etc/elasticsearch/jvm.options`
  * `<WHATEVER_IN_JVM_OPTIONS_FILE>` mostly includes heap size setup (e.g. `-Xms<A1>`, `-Xms<A2>`).


### Essential Configuration

see [here](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/settings.html#_config_file_location) for detail description

#### elasticsearch.yml
```yml
# one cluster can have several nodes up and running
cluster.name: my-application

# One node is one process, each node can have different configuration sets.
# Users can run several elasticsearch processes (depend on hardware computing
# capability), other running node(s) will detect a new node coming in.
node.name: my-node-125

# path to store persistent data (e.g. index, document storage, logs)
path.data: /PATH/TO/CONFIG_FOLDER/data/
path.logs: /PATH/TO/CONFIG_FOLDER/logs/

network.host: 127.0.0.1
http.port: 9201

# Lock the memory on startup
bootstrap.memory_lock: <true/false>

# 
```

#### jvm.options
```
# set heap size
-Xms128m
-Xmx128m
# everything else can be default values
```

Note:
* Why it is NOT recommended for a running elasticsearch instance to:
  * swap in memory space allocated by OS
  * swap out memory space to disk I/O
  
  because that could cause lots of [disk thrashing](https://www.computerhope.com/jargon/t/thrash.htm), which could degrade performance. [(reference #1)](https://stackoverflow.com/a/37608824/9853105) [(reference #2)](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/_memory_lock_check.html)
* Because of the degradation described above, it is suggested to [lock allocated memory to an elasticsearch instance](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/setup-configuration-memory.html#mlockall) when it starts.
  * In `/PATH/TO/ES_CONFIG/elasticsearch.yml`, set `bootstrap.memory_lock: true` to `true`
  * In `/etc/default/elasticsearch`, set `MAX_LOCKED_MEMORY` to `unlimited`
  * In `/etc/security/limits.conf`, add following lines, so the user `elasticsearch` and `<USER_OR_GROUP_NAME>` can enable memlock on  any process they run:
    ```
    <USER_OR_GROUP_NAME> soft memlock unlimited  
    <USER_OR_GROUP_NAME> hard memlock unlimited 
    elasticsearch soft memlock unlimited 
    elasticsearch hard memlock unlimited
    ```
  * In Ubuntu, you may need to relogin, for these settings to take effect.
  * You can check whether the memory lock is enabled, by calling API `/_nodes?filter_path=**.mlockall`
  * See more detail from [here](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/setup-configuration-memory.html#mlockall)
  
* You still need to WISELY configure your elasticsearch instance running in a machine which also hosts other services. [(reference #3)](https://stackoverflow.com/questions/37608486/using-mlockall-to-disable-swapping#comment84366798_37608824)
* it is better to set initial JVM heap size (`-Xms<A1>`) equal to its maximum size (`-Xmx<A2>`, in other words, `A1 == A2`), the size can be smaller than default setting, see [here](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/_heap_size_check.html#_heap_size_check) for reason.
* Elasticsearch is unstable [if heap space is too small](https://www.elastic.co/blog/a-heap-of-trouble), **you need to provide minimal required heap space that makes elasticsearch stable** (e.g. 128 MB should be safe):
  * because it cause more garbage collections (GC), each garbage collection pauses all working threads inside elasticsearch
  * for some reason, each garbage collection will take longer to complete
  * As soon as GC takes more than 1 second, all working threads have halted for more than 1 second, elasticsearch throws a Java exception named `OutOfMemoryError`, then the node goes down.
  * such problem exists, if your elasticsearch log file (default path `/var/log/elasticsearch`) contains many messages about GC like this (and logging time is close to each other) :
   ```
   [DATE_TIME][INFO][o.e.m.j.JvmGcMonitorService] [NODE_NAME] [gc][xxx] overhead, spent [322ms] collecting in the last [1s]
   [DATE_TIME][INFO][o.e.m.j.JvmGcMonitorService] [NODE_NAME] [gc][xxx] overhead, spent [459ms] collecting in the last [1s]
   [DATE_TIME][INFO][o.e.m.j.JvmGcMonitorService] [NODE_NAME] [gc][xxx] overhead, spent [479ms] collecting in the last [1s]
   [DATE_TIME][INFO][o.e.m.j.JvmGcMonitorService] [NODE_NAME] [gc][xxx] overhead, spent [352ms] collecting in the last [1s]
   [DATE_TIME][INFO][o.e.m.j.JvmGcMonitorService] [NODE_NAME] [gc][xxx] overhead, spent [306ms] collecting in the last [1s]
   [DATE_TIME][INFO][o.e.m.j.JvmGcMonitorService] [NODE_NAME] [gc][xxx] overhead, spent [527ms] collecting in the last [1s]
   [DATE_TIME][INFO][o.e.m.j.JvmGcMonitorService] [NODE_NAME] [gc][xxx] overhead, spent [283ms] collecting in the last [1s]
   [DATE_TIME][INFO][o.e.m.j.JvmGcMonitorService] [NODE_NAME] [gc][xxx] overhead, spent [328ms] collecting in the last [1s]
   ```


### Extra System Configuration
#### File Descriptor
If you start elasticsearch node as a stand-alone process (not starting as a service), be aware of maximum number of [file descriptors](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/file-descriptors.html#file-descriptors) allowed in the OS account. Because elasticsearch uses lots of file descriptors, **running out of file descriptors could probably lead to data loss**.

Elasticsearch node will abort on startup if test failure happens on [bootstrap check](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/bootstrap-checks.html#bootstrap-checks). Be sure to set up sufficient number of file descriptors on the OS account that run a elasticsearch node, by editing `/etc/security/limits.conf` :

```
<USER_OR_GROUP_NAME> -  nofile 65536
```

In Ubuntu, you need to relogin, for the setting to take effect.

Then recheck whether the setting is updated, by calling API endpoint `/_nodes/stats/process?filter_path=**.max_file_descriptors&pretty`, the result may be like:
```
{
  "nodes" : {
    "xxxxxxxxxx" : {
      "process" : {
        "max_file_descriptors" : 65536
      }
    }
  }
}
```

### x-pack
You may need to add a few options ONLY if x-pack plugin is installed 

In `elasticsearch.yml`
```
xpack.watcher.enabled: false

xpack.security.authc.accept_default_password : true
```

Note:
* If you start an elasticsearch node out of service, the node will report following error :
   ```
   java.lang.IllegalStateException: watch store not started
   ```
   By setting `xpack.watcher.enabled`, the [watcher](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/notification-settings.html) can be temporarily disabled in development mode. Note the error above doesn't show up  in elasticsearch node running as a service.
   
* Also x-pack will increase JVM heap usage, make sure there's enough memory space in JVM heap, it is better off having **at least** 128MB in initial heap space.
* Default password is `changeme` for all built-in user accounts e.g. `elastic`, `kibana`, `logstash_system`.
  * You can use any of these built-in users, the downsides are:
    * **the built-in users are fixed and CANNOT be changed by any other user** (even those who have superuser role), which makes them inconvenient to use
    * the 2 built-in users `logstash_system` and `kibana` still lack some [cluster privileges / indices privileges](https://www.elastic.co/guide/en/elasticsearch/reference/6.8/security-privileges.html) to make logstash / kibana work properly. 
  * therefore you are better off :
    * [creating another role](./basic_usage_cheatsheet.md#create-a-role) with all required cluster/indices privileges
      * for logstash, you should **at least** provide :
        * `manage_index_templates` in the cluster privilege list
        * add index pattern(s) to the indices access list, e.g. you may want logstash to flush log data with index name like `logstash-*`, `log-*`, or whatever patterns you attempt to use for indexing.
        * add indice privileges `create_index` and `index`, for every index pattern you add to the indices access list
        * Here is an example of the request body:
          ```json
          # POST /_xpack/security/role/logstash_role
          {
             "cluster" : ["manage_index_templates"],
             "indices" : [
                 {"names" : ["log-*", "logstash-*"], "privileges" : ["create_index", "index"]  },
                 {"names" : ["internal-*", "other-index-*"], "privileges" : ["create_index", "index"]  }
             ]
          }
          ```
      * for kibana, you should **at least** provide :
        * add index pattern(s) to the indices access list, as shwon in the example above
        * add indice privileges `read`, for every index pattern you add to the indices access list
        * Here is an example of the request body:
        ```json
        # POST /_xpack/security/role/kibana_role
        {
            "cluster" : [],
            "indices" : [
                {"names":["log-*", "logstash-*"], "privileges":["read"] },
                {"names":["internal-*", "other-index-*"], "privileges" : ["read"]}
            ]
        }
        ```
    * [creating another new user](./basic_usage_cheatsheet.md#create-user) and assigning the new role to the new user.
      * for logstash, you simply assign `logstash_role` to the role list of your new user
      * for kibana, you assign the built-in role `kibana_system` and the role you created `kibana_role` to the role list of your new user. Note that the built-in role `kibana_system` contains cluster privilege `monitor` which is required by kibana, so it's good to reuse it.
* you can [change the passwords](./basic_usage_cheatsheet.md#change-password) of an existing user account for security concern,
* Before production, remember to set false to the option `accept_default_password`.
* [Follow the steps](https://discuss.elastic.co/t/dec-22nd-2017-en-x-pack-i-lost-forgot-the-elastic-user-password-am-i-locked-out-forever/110075) while you forgot password of the built-in account (TODO)


### Reference
* [Official documentation v5.6](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/index.html)
