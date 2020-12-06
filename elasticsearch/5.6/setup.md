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
/ES_HOME/bin/elasticsearch -Epath.conf=/PATH/TO/CONFIG/FOLDER
```

Note
* `path.conf` exists only in old versions (probably before v6.0), removed in later version
* `/PATH/TO/CONFIG/FOLDER` contains few essential config files `jvm.options`, `log4j2.properties`, `elasticsearch.yml`.
* The default `ES_HOME` could be `/usr/share/elasticsearch` in Ubuntu/Linux system. 


### Configuration

see [here](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/settings.html#_config_file_location) for detail description

#### Important parameters (for quickstart)

* `elasticsearch.yml`
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
bootstrap.memory_lock: true

# 
```
Note:
* Why it is NOT recommended for a running elasticsearch instance to:
  * swap in memory space allocated by OS
  * swap out memory space to disk I/O
  
  because that could cause lots of [disk thrashing](https://www.computerhope.com/jargon/t/thrash.htm), which could lead to performance degrade. [(reference #1)](https://stackoverflow.com/a/37608824/9853105) [(reference #2)](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/_memory_lock_check.html)
* Based on statement above, it is suggested to [lock allocated memory to an elasticsearch instance, and check if the lock is successfully enabled](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/setup-configuration-memory.html#mlockall) when it starts.
* HOWEVER, you also need to WISELY configure if your elasticsearch instance runs in a machine which also includes other services. [(reference #3)](https://stackoverflow.com/questions/37608486/using-mlockall-to-disable-swapping#comment84366798_37608824)
* 


[Official documentation v5.6](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/index.html)
