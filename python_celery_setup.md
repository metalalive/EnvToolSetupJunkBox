#### Project setup

* start a project that includes celery
  * create folder ` <PATH/TO/PROJ/HOME>/proj123/init456`, change to the created folder
  * create `proj123/init456/celery.py` for celery app init
  * create `proj123/init456/celeryconfig.py` for configuation (e.g. register tasks for workers)
  * create `proj123/alltasks.py` for task functions workers would run. 


#### Initialization code

In `celery.py`:

```python
import os

from celery import Celery
from . import celeryconfig

# set default Django settings module for Celery application
os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'restaurant.settings')

app = Celery('whatever_app_label')
# programmatically load configuration module
app.config_from_object(celeryconfig)

if __name__ == '__main__':
    app.start()
```

#### Configuration Module

In `celeryconfig.py` :

```python
# explicitly indicate all tasks applied in this project
imports = ['proj123.alltasks']

# data transfer between clients (producers) and workers (consumers) needs to be serialized.
task_serializer = 'json'
result_serializer = 'json'

timezone = "Asia/Taipei"

# in my case, I use rabbitMQ as message broker. Use rabbitmqctl to manage accounts
broker_url = 'pyamqp://username:password@hostname//'

# relativa path is ok, be sure to enable write permission on the folder
result_backend = 'file://./tmp/celery/result'

# send result as transient message back to caller, not store it somewhere
# (e.g. database, file system)
#result_backend = 'rpc://'
# set False as transient message, if set True, then the message will NOT be
# lost after broker restarts.
# Official documentation mentions it is only for RPC backend, but actually it does not work
#result_persistent = False

# seperate queues for different tasks 
task_queues = {
    'queue_name_1'  : {'exchange':'queue_name_1',   'routing_key':'queue_name_1'},
    'queue_name_2'  : {'exchange':'queue_name_2',   'routing_key':'queue_name_2'},
    'queue_name_3'  : {'exchange':'queue_name_3',   'routing_key':'queue_name_3'},
}

task_routes = {
    'proj123.alltasks.func1':
    {
        'queue':'queue_name_1',
        'routing_key':'proj123.alltasks.func1',
    },
    'proj123.alltasks.func2':
    {
        'queue':'queue_name_2',
        'routing_key':'proj123.alltasks.func2',
    },
}

# set rate limit, at most 6 tasks to process in a single minute.
task_annotations = {
    'proj123.alltasks.func1': {'rate_limit': '10/m'},
    'proj123.alltasks.func2': {'rate_limit': '8/s'},
}

# following 3 parameters affects async result sent from a running task
task_track_started = True
# task_ignore_result = True
# result_expires , note the default is 24 hours

```

#### Task function Sample

In `alltasks.py`

```python
from proj123.init456.celery import app as celery_app

@celery_app.task(bind=True)
def func1(self, **kwargs):
    tsk_instance = self
    # Do something
    return 1
    
@celery_app.task
def func2(**kwargs):
    # Do something
    return 23
```

Note that :
* The argument `bind` in `celery_app.task` means the task function is bound with an instance of `celery.app.Task` class. By setting it to `True`, you can access the `celery.app.Task` instance inside the function (e.g. to give extra information, to implement progress bar of the running task) 



#### Start worker(s) to consume task requests from message queue (e.g. RabbitMQ or Redis)

##### Foreground

```
> cd <PATH/TO/PROJ/HOME>
> celery --app=proj123.init456  --config=proj123.init456.mycelerycfg  worker  --loglevel=INFO \
    --hostname=<YOUR_NODE_NAME>  -E -Q queue_name_1,queue_name_2,queue_name_3
```
note that :
  * `celery` utility will automatically find the file `celery.py` under the package `proj123/init456`.
  * `--config` allows you to specify the module path of celery configuration object (`proj123.init456.mycelerycfg`) at OS console, the alternative is to load configuration object programmatically by `Celery.config_from_object()` in your [initialization code](#initialization-code).
  * default queue name is `celery`, you can specify multiple queue name by adding option `-Q` with a list of queue name, also the queue names in the command above must match the name in your configuration module.
  * `--loglevel` can be `INFO`, `DEBUG`, `WARNING`
  * If you need to start several worker processes on the same physical host machine, make sure to set distinct node name `<YOUR_NODE_NAME>` in `--hostname` option for each worker process. Also `<YOUR_NODE_NAME>` is NOT related to the app label `whatever_app_label` you set in the [initialization code](#initialization-code)
  * If your tasks require Django library and you didn't set environment variable `DJANGO_SETTINGS_MODULE` in your [initialization code](#initialization-code), then you can also set `DJANGO_SETTINGS_MODULE` when running this command, for example :
    ```
    DJANGO_SETTINGS_MODULE='<YOUR_PATH_TO_DJANGO_SETTING_FILE>' celery --app=xxx5 --config=xxx4 worker -Q xxx1,xxx2,xxx3 ...
    ```

##### Background

Use [`start-stop-daemon`](https://man7.org/linux/man-pages/man8/start-stop-daemon.8.html) utility, it is convinient for unprivileged users who needs to run daemon processes. Celery 5.0 introduces `celery multi start/stop`, which doesn't work as expected, (in my case, celery workers daemon are NOT really created after running this command, it seems that nobody on any online community faced such issue)

Assume project path is at `<PATH/TO/YOUR/PROJ>/proj123`

* To start daemon:
```
start-stop-daemon --start --chuid <USER_NAME> --chdir <PATH/TO/YOUR/PROJ>  --background  --make-pidfile \
    --pidfile "celerydaemon.pid" --name celery_daemon   --user  <USER_NAME>  --exec <PATH/TO/YOUR/CELERY/BIN> \
    --  --app=proj123  --config=proj123.mycelerycfg  worker --loglevel=INFO  --hostname=<YOUR_NODE_NAME> \
        --logfile=./celerydaemon.log  -E -Q queue_name_1,queue_name_2,queue_name_3
```
Note that `--chuid` may be different from `--user` depending on application requirement.
After the command above, check PID from `<PATH/TO/YOUR/PROJ>/celerydaemon.pid` and log content from `<PATH/TO/YOUR/PROJ>/celerydaemon.log`

(TODO) how to dynamically set environment variable for Django settings file ?


* To stop daemon:
```
start-stop-daemon --stop --chuid <USER_NAME> --chdir <PATH/TO/YOUR/PROJ>  --make-pidfile --pidfile "celerydaemon.pid" \
    --user  <USER_NAME> --signal INT
```




#### Reference
* [Workers guide](https://docs.celeryproject.org/en/stable/userguide/workers.html)
* [next step](https://docs.celeryproject.org/en/latest/getting-started/next-steps.html)
* [configuration module](https://docs.celeryproject.org/en/stable/userguide/configuration.html#std-setting-imports)
* [daemonization](https://docs.celeryproject.org/en/stable/userguide/daemonizing.html)
* [Good practices](https://denibertovic.com/posts/celery-best-practices/)
* [django/celery: Best practices to run tasks on 150k Django objects?](https://stackoverflow.com/questions/7493306/django-celery-best-practices-to-run-tasks-on-150k-django-objects)

