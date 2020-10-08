#### Manage login account

Note that the commands below have to be executed by privileged user (superuser)

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

* permissions

Permission must be granted before the user login the broker with the account,
 (what are other options ?)

```
// grant full permission (read, write, execute)
rabbitmqctl set_permissions -p /  runfaster ".*" ".*" ".*"
```




