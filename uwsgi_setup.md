#### uWSGI, WSGI, and PEP3333

##### Environment

* OpenSSL 1.1.1c
* Python 3.9.05a
* uWSGI 2.0.18
all these software above are built from source


##### Build

* Download uWSGI from github
* `PyEval_CallObject()` and `PyEval_InitThreads()` is deprecated since `python 3.9`, if  there's any deprecated C functions in the C source code of your uWSGI repository. You can :
   * replace `PyEval_CallObject()` with another recommended function `PyObject_CallObject()`
   * simply remove `PyEval_InitThreads()`, which will do nothing since `python 3.9`
* For those who built python from source, specify python library source by :
   * add `libdir = "/PATH/TO/YOUR/PYTHON/SRC/HOME"` in `plugins/python/uwsgiplugin.py`
   ```Shell
   libdir = /PATH/TO/YOUR/PYTHON/SRC/HOME
   libpath = '%s/libpython%s.a' % (libdir, version)
   ```
   * build uWSGI with specified python version (e.g. in my case, python 3.9):
   ```Shell
   make all  PYTHON=/PATH/TO/YOUR/PYTHON/SRC/HOME/python
   ```



##### Test

* Example Python application to hook up wsgi middleware

```Python
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


* command to launch uWSGI
  * don't run it with root privilege
  * each `uwsgi` instance can only bind one application, for hosting multiple applications simultaneously, run multiple uWSGIs instead.

```Tcsh
./uwsgi --http 127.0.0.1:8006 --virtualenv  PATH/TO/YOUR/VIRTUALENV \
    --wsgi-file  PATH/TO/YOUR/PYTHON/APP  --enable-threads --processes 1  --threads 1
```

#### Run your application with config file

Assume your config file `xxx.ini` looks like this :
```Windows Registry Entries
[uwsgi]
http-socket = 127.0.0.1:8005
virtualenv  = PATH/TO/YOUR/VIRTURLENV
pythonpath = PATH/TO/YOUR/PYTHON/SRCCODE
wsgi-file  = YOUR_WEB_APP_ENTRY.py
enable-threads = true
master    = true
processes = 1
workers   = 2
pidfile   = pid.log
```

you have :
```
./uwsgi --ini=xxx.ini >& runtime.log &
```



#### Reference

* [PEP3333](https://www.python.org/dev/peps/pep-3333/)
* [uWSGI options](https://uwsgi-docs.readthedocs.io/en/latest/Options.html)
* [Deploy Django behind uWSGI and Nginx](https://www.vndeveloper.com/deploy-django-in-sub-directory-behind-uwsgi-and-nginx-on-centos-7/?fbclid=IwAR109_JIrhh_gssvbPvJ8FM6smBBW4w8bLxawx1dv9SoLauZLCf0z7JOMGI)


