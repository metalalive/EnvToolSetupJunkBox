## ELK setup on Ubuntu

Ubuntu version : 14.04 LTS

| Software      | Version       |
| ------------- | ------------- |
| Elasticsearch | 5.0.0         |
| Logstash      | 5.6.8         |
| Kibana        | 5.0.0         |
| Java          | 1.8.0_222     |

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

* install (headless version of JRE should be sufficient to run ELK)
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

##### verify the installed package by running the command below.

note that :
  * setting file is required for `logstash` command, for beginners you can use default setting file which is located at `/etc/logstash`
  * default setting file (in `/etc/logstash`) will require access permission to `/var/log/logstash`, if permission requirement doesn't meet, it will report error at JVM level.
  * you need another configuration file to specify input (where to receive messages, for testing purpose it can be stdin) and output (where to forward the received messages, for testing purpose it can be stdout)


sample code in `/YOUR/PATH/TO/sample.conf`
```
input { 
  stdin { } 
}
output {
 stdout { codec => rubydebug }
}
```

Commands :
```
sudo /usr/share/logstash/bin/logstash -f /YOUR/PATH/TO/sample.conf --path.settings /etc/logstash
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


