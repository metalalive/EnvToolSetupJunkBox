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
  * `<WHATEVER_IN_JVM_OPTIONS_FILE>` mostly includes heap size setup.


### Configuration

see [here](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/settings.html#_config_file_location) for detail description

#### Important parameters (for quickstart)

##### elasticsearch.yml
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

##### jvm.options
```
# set heap size
-Xms64m
-Xmx64m
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


[Official documentation v5.6](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/index.html)
