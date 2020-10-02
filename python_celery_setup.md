#### Project setup

* start a project that includes celery
make dir ` <PATH/TO/PROJ/HOME>/proj123` , create `celery.py` for celery app init,  `celeryconfig.py` for configuation (e.g. register tasks for workers), `alltasks.py` for task functions workers would run. 


#### Start worker(s) to consume task requests from message queue (e.g. RabbitMQ or Redis)

##### Foreground

```
cd <PATH/TO/PROJ/HOME>

celery --app=proj123  worker  --loglevel=INFO
```
note `celery` utility will automatically find out default queue named `celery.py` under `proj123` project folder.


##### Background

Use `start-stop-daemon` utility, it is convinient for unprivileged users who needs to run daemon processes. Celery 5.0 introduces `celery multi start/stop`, which doesn't work as expected, (in my case, celery workers daemon are NOT really created after running this command, it seems that nobody on any online community faced such issue)

Assume project path is at `<PATH/TO/YOUR/PROJ>/proj123`

* To start daemon:
```
start-stop-daemon --start --chuid <USER_NAME> --chdir <PATH/TO/YOUR/PROJ>  --background  --make-pidfile \
    --pidfile "celerydaemon.pid" --name celery_daemon   --user  <USER_NAME>  --exec <PATH/TO/YOUR/CELERY/BIN> \
    --  --app=proj123 worker --loglevel=INFO --logfile=./celerydaemon.log
```
Note that `--chuid` may be different from `--user` depending on application requirement.
After the command above, check PID from `<PATH/TO/YOUR/PROJ>/celerydaemon.pid` and log content from `<PATH/TO/YOUR/PROJ>/celerydaemon.log`


* To stop daemon:
```
start-stop-daemon --stop --chuid <USER_NAME> --chdir <PATH/TO/YOUR/PROJ>  --make-pidfile --pidfile "celerydaemon.pid" \
    --user  <USER_NAME> --signal INT
```




#### Reference
* [next step](https://docs.celeryproject.org/en/latest/getting-started/next-steps.html)
* [configuration module](https://docs.celeryproject.org/en/stable/userguide/configuration.html#std-setting-imports)
* [daemonization](https://docs.celeryproject.org/en/stable/userguide/daemonizing.html)

