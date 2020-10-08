#### Manage login account and permissions

Note that the commands below have to be executed by privileged user (superuser)

```
rabbitmqctl add_user <USERNAME> <PASSWORD>
rabbitmqctl list_users
rabbitmqctl clear_password <USERNAME>
rabbitmqctl change_password <USERNAME> <PASSWORD>


```



