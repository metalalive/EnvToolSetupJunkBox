### uWSGI, WSGI, and PEP3333

#### Environment

* OpenSSL 1.1.1c
* Python 3.9.05a
* uWSGI 2.0.18
all these software above are built from source


#### Build

##### preparation
* Download uWSGI from github
* `PyEval_CallObject()` and `PyEval_InitThreads()` is deprecated since `python 3.9`, if  there's any deprecated C functions in the C source code of your uWSGI repository. You can :
   * replace `PyEval_CallObject()` with another recommended function `PyObject_CallObject()`
   * simply remove `PyEval_InitThreads()`, which will do nothing since `python 3.9`

##### Build with python plugin, or language independent
There are 2 ways of building uwsgi binary :
1. build with python plugin, so your uwsgi binary will be tied to specific python version
2. build language-independent binary first, and then build python plungin for different python versions.
  * the second option does NOT seem to work for python intepreter built from source (in my case, python3.9), but the workaround is to follow the first option, build several uwsgi binaries, each of which is tied to specific python version. It works well **due to the fact that a built uwsgi binary is actually a standalone executable file**

The first option above can be further split into :
* build using `make` command
  For those who build python from source, specify python library source by :
  * add `libdir = "/PATH/TO/YOUR/PYTHON/SRC/HOME"` in `plugins/python/uwsgiplugin.py`
    ```Shell
    libdir = /PATH/TO/YOUR/PYTHON/SRC/HOME
    libpath = '%s/libpython%s.a' % (libdir, version)
    ```
  * build uWSGI with specified python version (e.g. in my case, python 3.9):
    ```Shell
    make all  PYTHON=/PATH/TO/YOUR/PYTHON/SRC/HOME/python
    ```
* build using specific version of python intepreter, which is easier and less error-prone 
  ```shell
  /PATH/TO/YOUR/PYTHON/SRC/HOME/python  uwsgiconfig.py --build
  ```

After building successfully, you should see the binary named `uwsgi` located at `/PATH/TO/YOUR/UWSGI/FOLDER`




#### Test Sample

* Example Python application to hook up wsgi middleware

```Python
# By default, wsgi looks for callable object named `application()`
# and run it as an entry point of your python application 
def application(env, start_resp_cb):
    # check out more environment variables supported in PEP-3333
    path = env.get('PATH_INFO')
    if path == '/halo':
        resp_body = 'hallow'
    elif path == '/soul':
        resp_body = 'solo'
    else:
        resp_body = 'not greeting'
    resp_body = resp_body.encode("utf-8")
    status = '200 OK'
    # In wsgi, the response header is a list of (header_name, header_value) tuple,
    # here we only have one tuple. In Python it must be a list type
    resp_header = [("Content-Length", str(len(resp_body)))]
    print("[application] receive request {}, response: {}".format(str(path), resp_body ))
    # applications must invoke response callable (callback function),
    # make wsgi middleware send the response header bytestring prior to
    # sending bytes of response body content (e.g. HTML/XML)
    # It must return a write(body_data) callable.
    # TODO: figure out what application developers can do with this returned write
    #       callable.
    start_resp_cb(status, resp_header)
    # return list of bytestrings as response body, let wsgi middleware pass them to
    # (http) web server or client
    return [resp_body]
```

#### Run
* command to launch uWSGI
  * don't run it with root privilege
  * each `uwsgi` instance can only bind one application, for hosting multiple applications simultaneously, run multiple uWSGIs instead.
  * you can put all arguments in the command line console, or gather them to separate configuration file (recommended) 

```Tcsh
./uwsgi --http 127.0.0.1:8006 --virtualenv  PATH/TO/YOUR/VIRTUALENV \
    --wsgi-file  PATH/TO/YOUR/PYTHON/APP  --enable-threads --processes 1  --threads 1
```

Note that configuration file can be ini, xml, and json format, see [this section](https://uwsgi-docs.readthedocs.io/en/latest/Configuration.html#ini-files) for more detail.

Assume your config file `xxx.ini` looks like this :
```Windows Registry Entries
[uwsgi]
...

[section123]
http = 127.0.0.1:8081
socket = :8082
chdir = /PATH/TO/YOUR/PYTHON/PROJECT_HOME
virtualenv  = /PATH/TO/YOUR/VIRTURLENV
pythonpath = /PATH/TO/YOUR/PYTHON/SRCCODE
wsgi-file  = /PATH/TO/YOUR_WEB_APP_ENTRY.py
module   = PYTHON_PACKAGE.TO_YOUR_MODULE:APPLICATION_CALLABLE
env = ENV_VAR_1=VALUE1  ENV_VAR_2=VALUE2 ....
enable-threads = true
master    = true
processes = 1
workers   = 2
pidfile   = /PATH/TO/pid.log
daemonize = /PATH/TO/YOUR_LOG_FILE
```

you have :
```
./uwsgi --ini=xxx.ini:section123 >& runtime.log &
```

Note:
* you have to either provide `http` (or `https`) or `socket` option to accept incoming request
  * if `http`/`https` is set, then uwsgi instance will run as a web server with http/https protocol, you can then test it with frontend tools (e.g. curl, web browsers)
  * if `socket` is set, then uwsgi instance will run as a server with WSGI protocol. In such case you will need extra http server like Apache or Nginx to hook with the running uwsgi instance.
* you have to either provide `wsgi-file`  or `module` option as the entrypoint of your WSGI application
  * for `wsgi-file` , the default entrypoint is a python callable object named `application`
  * for `module` , the entrypoint is named as the same as `APPLICATION_CALLABLE` in the python module `PYTHON_PACKAGE.TO_YOUR_MODULE`
* `env` is a list of key-value pairs as OS environment variables, delimited by whitespace
* `virtualenv` indicate the path to your python virtualenv folder, if you built uwsgi binary tied to specific python version, you have to ensure **the python version used in your virtualenv** matches **the python version used to build your uwsgi binary**, otherwise you'll hit [this error](https://stackoverflow.com/questions/56443552)
* `pidfile` only record main process of the running uwsgi instance, if you want to terminate the running uwsgi instance in OS, you will have to send SIGINT to the process whose ID is stored in `pidfile`, see [signals for controlling uWSGI](https://uwsgi-docs.readthedocs.io/en/latest/Management.html?highlight=SIGINT#signals-for-controlling-uwsgi)
* `daemonize` will make a uwsgi instance run as a daemon, without this option a uwsgi instance runs as foreground process(es).


#### Reference

* [PEP3333](https://www.python.org/dev/peps/pep-3333/)
* [uWSGI options](https://uwsgi-docs.readthedocs.io/en/latest/Options.html)
* [Deploy Django behind uWSGI and Nginx](https://www.vndeveloper.com/deploy-django-in-sub-directory-behind-uwsgi-and-nginx-on-centos-7/?fbclid=IwAR109_JIrhh_gssvbPvJ8FM6smBBW4w8bLxawx1dv9SoLauZLCf0z7JOMGI)
* [Setting up Django and your web server with uWSGI and nginx](https://uwsgi-docs.readthedocs.io/en/latest/tutorials/Django_and_nginx.html)

