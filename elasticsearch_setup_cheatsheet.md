### Run
#### run it as a service

```
service elasticsearch start
service elasticsearch stop
```

#### run it as a stand-alone process
```
/ES_HOME/bin/elasticsearch -Epath.conf=/PATH/TO/CONFIG/FOLDER
```

Note
* `path.conf` exists only in old versions (probably before v6.0), removed in later version
* `/PATH/TO/CONFIG/FOLDER` contains few essential config files `jvm.options`, `log4j2.properties`, `elasticsearch.yml`. see [here](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/settings.html#_config_file_location) for detail description.
* The default `ES_HOME` could be `/usr/share/elasticsearch` in Ubuntu/Linux system. 


[Official documentation v5.6](https://www.elastic.co/guide/en/elasticsearch/reference/5.6/index.html)
