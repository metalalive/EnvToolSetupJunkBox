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

#### Important parameters

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
```



[Official documentation v5.6](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/index.html)
