#### [Manage login account](https://www.rabbitmq.com/access-control.html#user-management)

Note that **ALL the commands** below have to be executed by privileged user (superuser)

* Create username / password
```
rabbitmqctl add_user <USERNAME> <PASSWORD>
rabbitmqctl clear_password <USERNAME>
rabbitmqctl change_password <USERNAME> <PASSWORD>
```

* list all users
```
rabbitmqctl list_users
```


* `set_user_tags` is NOT officially clearly documented, I do not know any other options besides `administrator`

```
rabbitmqctl set_user_tags <USERNAME> administrator
rabbitmqctl set_user_tags <USERNAME> // clean up all tags added on the user
```

#### Permissions, see [`set_permission` option](https://www.rabbitmq.com/rabbitmqctl.8.html)

Permission must be granted before the user login the broker with the account
```
// grant full permission (read, write, execute)
rabbitmqctl set_permissions -p /  runfaster ".*" ".*" ".*"
```
You can also specify routing key for the user to read / write message, by adding regular expression to `read` field or `write` field
```
```


#### Queue operations

* list all queues available
the output format is : `<QUEUE_NAME>` followed by an integer that represents number of unprocessed messages / tasks

```
> rabbitmqctl list_queues
queue_name_1   0
queue_name_2   8
queue_name_3   127
>
```


#### Management web UI

* To enable the UI, run this command first then restart rabbitmq server
```
> rabbitmq-plugins  enable  rabbitmq_management
The following plugins have been enabled:
  mochiweb
  webmachine
  rabbitmq_web_dispatch
  amqp_client
  rabbitmq_management_agent
  rabbitmq_management
Plugin configuration has changed. Restart RabbitMQ for changes to take effect.
>
```
* To disable management UI :
```
> rabbitmq-plugins  disable  rabbitmq_management
```

The downside is that you must restart rabbitmq server for the change to take effect.


* Go to `http://localhost:15672` in web browser, login with the account you created, (be aware of the permission, or just set administrator user tag using `set_user_tags` )

* once you are logged-in, you can :
  * clean up the content  (purge) in any queue, or delete entire queue
  * see traffic history



#### Reference 
* [`rabbitmqctl(8)` documentation](https://www.rabbitmq.com/rabbitmqctl.8.html)
* [Access ans permissions](https://www.rabbitmq.com/management.html#permissions)
* [Management Plugin](https://www.rabbitmq.com/management.html)

