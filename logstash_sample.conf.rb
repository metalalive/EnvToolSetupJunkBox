## logstash configuration options

input {
    stdin {
        codec => "line"
    } 
    ## tcp {
    ##     port => 5959
    ##     codec => "json"
    ##     ssl_enable => false
    ## } 
} ## end of input phase


filter {
    grok {
        break_on_match => true
        named_captures_only => true
        match => {
            "message" => [
                ## comment, for development
                "%{IP:[request][ip]} %{WORD:[request][mthd]} %{URIPATH:[request][uri][path]}(?:%{URIPARAM:[request][uri][params]})? %{HOSTNAME:[code][module_path]} %{INT:[code][lineno]} %{LOGLEVEL:level} %{GREEDYDATA:raw_key_value_pairs}",
                ## comment, for logging user activity
                "%{IP:[request][ip]} %{WORD:[reqeust][mthd]} %{URIPATH:[request][uri][path]} %{INT:[profile][id]} %{WORD:[profile][firstname]} %{WORD:[affected][model_cls]}"
            ]
        }
    }

    ## filters can be chained like this
    if [raw_key_value_pairs] {
        kv {
            source => "raw_key_value_pairs"
            target => "dict_data"
        }
        if "_grokparsefailure" not in [tags] {
            mutate {
                remove_field => ["raw_key_value_pairs"]
            }
        }
    }

    if "_grokparsefailure" not in [tags] {
        mutate {
            remove_field => ["message"]
        }
    } ## keep unparsed message & ship to output phase
} ## end of filter phase


output {
    ## default, for quick test
    stdout { codec => rubydebug }

    ## ## eventually parsed message will go here, for storage
    ## elasticsearch {
    ##     hosts => ["localhost:9200"]
    ##     ## better to dynamically create index (e.g. by date, by application label)
    ##     index => "testlog"
    ##     sniffing => true
    ## }
} ## end of output phase


## [example input data]
## 12.34.109.2 GET /usrmgt/activity_log?abc=ty427yt248&ordering=usr_cnt softdelete.views 188 WARNING cicd=0008  pid= itu  haja = jacker  pid = 093
##
## [output]
## 
## {
##        "request" => {
##         "mthd" => "GET",
##          "uri" => {
##             "params" => "?abc=ty427yt248&ordering=usr_cnt",
##               "path" => "/usrmgt/activity_log"
##         },
##           "ip" => "12.34.109.2"
##     },
##     "@timestamp" => 2020-11-30T16:11:54.915Z,
##           "code" => {
##         "module_path" => "softdelete.views",
##              "lineno" => "188"
##     },
##          "level" => "WARNING",
##      "kv_target" => {
##         "haja" => "jacker",
##         "cicd" => "0008",
##          "pid" => [
##             [0] "itu",
##             [1] "093"
##         ]
##     },
##       "@version" => "1",
##           "host" => "0.0.0.0"
## }
##
## [reference]
## https://www.elastic.co/guide/en/logstash/current/event-dependent-configuration.html#logstash-config-field-references
## https://github.com/logstash-plugins/logstash-patterns-core/blob/master/patterns/grok-patterns

