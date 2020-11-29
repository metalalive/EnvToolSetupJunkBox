## ELK setup on Ubuntu

Ubuntu version : 14.04 LTS

| Software      | Version       |
| ------------- | ------------- |
| [Elasticsearch](#elasticsearch) | 5.6.16        |
| [Logstash](#logstash)      | 5.6.8         |
| [Kibana](#kibana)        | 5.6.16         |
| [Java](#java-8-open-jdk)          | 1.8.0_222     |

### Installation

#### Java 8 (Open JDK)

* make sure repository for java 8 is added, otherwise, run the command :
```
> sudo add-apt-repository  ppa:openjdk-r/ppa -y
```

* then update , check the version
```
> apt-get update
> apt-cache policy openjdk-8-jre-headless
```

* install, headless version of JRE should be sufficient to run ELK, no need to install JDK (`javac`)
```
> sudo apt-get install openjdk-8-jre-headless -y
```

* adjust default version, if there are multiple versions of JRE in your Ubuntu OS
```
> sudo update-alternatives --config java
```

* double-check JRE version
```
> java -version
openjdk version "1.8.0_222"
OpenJDK Runtime Environment (build 1.8.0_222-8u222-b10-1~14.04-b10)
OpenJDK 64-Bit Server VM (build 25.222-b10, mixed mode)
```


#### Logstash

There are 2 options for Logstash installation

##### Install by apt-get
* install from `apt-get`,please [read this article](https://fabianlee.org/2017/05/01/elk-installing-logstash-on-ubuntu-14-04/) <-- however I get stuck at **0% connection progress** when running `apt-get install logstash` (even I followed the steps described there carefully)

##### Manual installation
* alternative, manually download from [here](https://www.elastic.co/downloads/past-releases/logstash-5-6-8), select any of version 5.x.x, then run the installation command:
```
sudo dpkg -i logstash-5.6.8.deb
```

* double-check the installation afterwards:
```
dpkg -l | grep logstash

ls /usr/share/logstash
```

##### verify the installed package 

by running the command below.

note that :
  * setting file is required for `logstash` command, for beginners you can use default setting file which is located at `/etc/logstash`
  * default setting file (in `/etc/logstash`) will require access permission to `/var/log/logstash`, if permission requirement doesn't meet, it will report error at JVM level.
  * you need another configuration file to specify input (where to receive messages, for testing purpose it can be stdin) and output (where to forward the received messages, for testing purpose it can be stdout)

Commands :
```
sudo /usr/share/logstash/bin/logstash -f /YOUR/PATH/TO/sample.conf --path.settings /etc/logstash
```

where the sample code in `/YOUR/PATH/TO/sample.conf`.
in this case, every single line you typed is wrapped to certain well-defined log structure in logstash , then output to stdout
```
input { 
  stdin { } 
}
output {
 stdout { codec => rubydebug }
}
```

Result:
```
Sending Logstash's logs to /var/log/logstash which is now configured via log4j2.properties
The stdin plugin is now waiting for input:

master everything
{
      "@version" => "1",
          "host" => "0.0.0.0",
    "@timestamp" => 2020-11-28T08:40:02.820Z,
       "message" => "master everything"
}

what next ?
{
      "@version" => "1",
          "host" => "0.0.0.0",
    "@timestamp" => 2020-11-28T08:40:18.875Z,
       "message" => "what next ?"
}

```


#### Elasticsearch

##### Installation

* manually download from [here](https://artifacts.elastic.co/downloads/elasticsearch/elasticsearch-5.6.16.deb), then run the installation command:
```
sudo dpkg -i elasticsearch-5.6.16.deb
```

* double-check the installation afterwards:
```
dpkg -l | grep elasticsearch

ls /usr/share/elasticsearch
```

during installation, it may report error at the end like this:
```
Selecting previously unselected package elasticsearch.
(Reading database ... 239434 files and directories currently installed.)
Preparing to unpack elasticsearch-5.6.16.deb ...
Creating elasticsearch group... OK
Creating elasticsearch user... OK
Unpacking elasticsearch (5.6.16) ...
Setting up elasticsearch (5.6.16) ...
Failed to issue method call: Unit systemd-sysctl.service failed to load: No such file or directory. See system logs and 'systemctl status systemd-sysctl.service' for details.
Processing triggers for ureadahead (0.100.0-16) ...
```

haven't figured that out, but seems harmless.

##### Quick-start configuration

2 files to edit:

* `/etc/elasticsearch/elasticsearch.yml`
  * uncomment `cluster.name` and set to whatever name you like **without quotation mark**
    ```
    # correct
    cluster.name: my-application
    # incorrect
    cluster.name: "my-application"
    ```
  * uncomment `node.name` and set to whatever name you like
  * uncomment `network.host` and `http.port`, set your IP address (e.g. 127.0.0.1) and port (port default : 9200)
  

* `/etc/elasticsearch/jvm.options`
  * modify `-Xms2g` to `-Xms256m` (also `-Xmx2g` to `-Xmx256m`) to avoid elasticsearch from taking too much memory, do this ONLY if you don't have sufficient memory to allocate to JVM heap.


##### verify the installed package

turn on/off the service

```
service elasticsearch start
service elasticsearch stop
```

command to verify whether elasticsearch server is up, by retrieving its general information

```
curl --request GET http://localhost:9200
```

expected result:

```
{
  "name" : "my-node-name-123",
  "cluster_name" : "my-application",
  "cluster_uuid" : "<RANDOM_GENERATED_NUMBER>",
  "version" : {
    "number" : "5.6.16",
    "build_hash" : "xxxxxxx",
    "build_date" : "2019-03-13T15:33:36.565Z",
    "build_snapshot" : false,
    "lucene_version" : "6.6.1"
  },
  "tagline" : "You Know, for Search"
}

```


#### Kibana

##### Installation

Similar to the installation steps in [logstash](#logstash) and [elasticsearch](#elasticsearch) , except the [download link](https://artifacts.elastic.co/downloads/kibana/kibana-5.6.16-amd64.deb) is different, and Kibana doesn't require Java Runtime Environment.


##### Quick-start configuration
**Note** : this configuration does not include authentication / permission settings

Add following configuation options :
* `server.port: <PORT_NUMBER>` , 5601 by default
* `server.host: "<YOUR_HOSTNAME>"`, localhost by default
* `server.name: "<YOUR_SERVERNAME>"`
* `elasticsearch.url: <YOUR_ES_URL>`, `http://localhost:9200` by default


##### verify the installed package

Open browser and navigate to kibana front-end  (default `http://localhost:5601`), Kibana will redirect to the page with title "Configure an index pattern".
You will need to specify default index pattern, the default index pattern could start with `logstash-*` for Kibana to identify elasticsearch index then run search and analytics .

Simply press `create` button, you'll set default index pattern 





-----------------------------

#### Verify all together

Turn on service elasticsearch and kibana.

Logstash command is almost the same, except different sample configuation file
```
sudo /usr/share/logstash/bin/logstash -f /YOUR/PATH/TO/sample2.conf --path.settings /etc/logstash
```

where the sample code in `/YOUR/PATH/TO/sample2.conf`.
in this case, every line you typed is sent to elasticsearch server and store in there.
```
input { 
  stdin { } 
}
output {
    elasticsearch {
        hosts => ["localhost:9200"]
    }
}

```

Result: type any string
```
where Do we GO
where do we go NOW
```

Recheck whether elasticsearch received the message you typed above , by fetching data through elasticsearch RESTful API :

<pre><code>
curl -H "Content-Type: application/json" -H "Accept: application/json; indent=4;" \
    -H "X-ANTI-CSRF-TOK: xxiixixxo"    --request GET \
    http://localhost:9200/logstash-*/_search

{"took":2,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},
    "hits":{"total":2,"max_score":1.0, "hits":[
        {"_index":"logstash-2020.11.29","_type":"logs","_id":"xxxxxx-xxx","_score":1.0,"_source":{
            "@version":"1","host":"0.0.0.0","@timestamp":"2020-11-29T03:55:44.050Z","message":"where Do we GO"
        }},
        {"_index":"logstash-2020.11.29","_type":"logs","_id":"xxxxxxxxx-xxxx","_score":1.0,"_source":{
            "@version":"1","host":"0.0.0.0","@timestamp":"2020-11-29T04:00:55.799Z","message":"where do we go NOW"
        }}
    ]
}}

</code></pre>

the visible fields of returned structure depends on your configuration, but you can expect to see the message you typed previously in logstash command.


