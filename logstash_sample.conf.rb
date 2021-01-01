## logstash configuration options
input { 
    tcp {
        port => 5959
        codec => "line"
        ssl_enable => false
    }
} # end of input phase


filter {
    fingerprint {
        source => "message"
        target => "[@metadata][fingerprint]"
        # for C10K problem, maximum number of the log messages per day are
        # 3600 * 24 * 10000 * k, where k is number of log messages per frontend request
        # ,if k = 100, the number above is around 2^(36.3303...), so generated value for
        # the fingerprints unlikely collide (in just one day)
        method => "SHA1"
        key => "xxx"
    }

    grok {
        break_on_match => true
        named_captures_only => true
        patterns_dir => ['/etc/logstash/custom_pattern']
        match => {
            "message" => [
                ## comment, for development use
                ## URI parameters, process ID, and thread ID are optional
                ## ISO8601 timestamp format --> YYYY-MM-DD hh:mm:ss,uuu
                "(%{IP:[request][ip]} %{WORD:[request][mthd]} %{URIPATH:[request][uri][path]}(?:%{URIPARAM:[request][uri][params]})? )?%{TIMESTAMP_ISO8601:asctime} %{LOGLEVEL:level} %{INT:process} %{INT:thread} %{PATH:[code][filepath]} %{INT:[code][lineno]} %{GREEDYDATA:serial_json_msg}",
                ## comment, for logging user activity
                "%{IP:[request][ip]} %{WORD:[reqeust][mthd]} %{URIPATH:[request][uri][path]} %{INT:[profile][id]} %{WORD:[profile][firstname]} %{WORD:[affected][model_cls]}"
            ]
        }
    } ## end of grok plugin

    #date {
    #    ####match => ["[@metadata][timestamp]", "MMM dd yyyy HH:mm:ss", "ISO8601"]
    #    match => ["[@metadata][timestamp]", "yyyy-MM-dd-HH-mm-ss"]
    #    timezone => "Asia/Taipei"
    #}

    grok {
        # extract year and month from input event time (the app server)
        # for internal use, all fields in @metadata are not part of event at output time.
        match => {
            "asctime" => [
                "%{YEAR:[@metadata][evt_date][year]}-%{MONTHNUM:[@metadata][evt_date][month]}-%{GREEDYDATA:[@metadata][evt_date][useless]}"
            ]
        }
    }

    mutate {
        remove_field => ["[@metadata][evt_date][useless]"]
    }

    ## filters can be chained like this
    if [serial_json_msg] {
        #kv 
        json {
            source => "serial_json_msg"
            target => "msg_kv_pairs"
            #value_split => "="
            #field_split => "&"
        }
        if "_jsonparsefailure" not in [tags] and [msg_kv_pairs] {
            mutate {
                remove_field => ["serial_json_msg"]
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
    elasticsearch {
        hosts => ["YOUR_HOST_NAME:PORT"]
        user => "LOGIN_USERNAME"
        password => "LOGIN_PASSWORD"
        action => "index"
        # log indexes are separate by months
        # NOTE: DO NOT read event time by using syntax %{+yyyy.MM.dd.HH.mm}
        # because I have not figured out how to change the timezone (it might be impossible to change that) 
        #### index => "log-%{+yyyy.MM.dd.HH.mm}"
        index => "log-%{[@metadata][evt_date][year]}-%{[@metadata][evt_date][month]}"
        document_type => "app_server" # deprecated in v 7.x
        document_id   => "%{+dd}%{[@metadata][fingerprint]}"
    }
} # end of output phase


# [reference]
# https://www.elastic.co/guide/en/logstash/current/event-dependent-configuration.html#logstash-config-field-references
# https://github.com/logstash-plugins/logstash-patterns-core/blob/master/patterns/grok-patterns
# https://www.elastic.co/guide/en/logstash/current/event-dependent-configuration.html#event-dependent-configuration
# https://www.elastic.co/guide/en/logstash/current/plugins-filters-date.html#plugins-filters-date
#

        
