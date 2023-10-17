
### Python 3.12
- Tested on Ubuntu 14.04LTS & Debain 9 (Raspbian Stretch)
- OpenSSL 3.1.3 (version `3.x` required since python 3.11 , see [python developer guide](https://devguide.python.org/getting-started/setup-building/))
  - Both of source folder and installation folder are required

* Install dependency packages
```
sudo apt-get install -y build-essential tk-dev libncurses5-dev libncursesw5-dev libreadline6-dev \
             libdb5.3-dev libgdbm-dev libsqlite3-dev libssl-dev libbz2-dev libexpat1-dev \
             liblzma-dev zlib1g-dev libffi-dev tar wget vim
```

* Download [Python source code](https://github.com/python/cpython) & Go to the downloaded Python source directory.
> Please avoid **ALPHA release or BETA release**,
> always use **FINAL release** instead, you will save a lot of time.

* Configure everything required (you might need root priviledge to do things). For those who built OpenSSL from source for whatever reasons, it is required to specify the path to your openssl installation / codebase, with the options `--with-openssl`, `--with-openssl-rpath`, `CFLAGS` and `LDFLAGS`  when running `./configure` below:

```
./configure --enable-optimizations --with-ssl-default-suites=openssl \
    --with-openssl=/PATH/TO/YOUR/OPENSSL/CODEBASE/ \
    --with-openssl-rpath=/PATH/TO/YOUR/OPENSSL/INSTALLED/lib64 \
    CFLAGS="-I/PATH/TO/YOUR/OPENSSL/INSTALLED/include" \
    LDFLAGS="-L/PATH/TO/YOUR/OPENSSL/INSTALLED/lib64"
```


* Check out `config.log`. For those who built OpenSSL from source, make sure you get following results :
```
checking for --with-ssl-default-suites... openssl
...
checking for include/openssl/ssl.h in /PATH/TO/YOUR/OPENSSL/CODEBASE/... yes
checking whether compiling and linking against OpenSSL works... yes
...
checking whether OpenSSL provides required ssl module APIs... yes
checking whether OpenSSL provides required hashlib module APIs... yes
...
checking for stdlib extension module _ssl... yes
checking for stdlib extension module _hashlib... yes
```

* start building
```
make build_all -j 1 >& build.log
```

* Recheck build.log. For those who built OpenSSL from source, make sure you DON'T have the following message in your `build.log`, otherwise you will get secure connection failures later when you try to download packages through `pip`.
```
Failed to build these modules:
_ssl
.....
Could not build the ssl module!
Python requires an OpenSSL 1.0.2 or 1.1 compatible libssl with X509_VERIFY_PARAM_set1_host().
LibreSSL 2.6.4 and earlier do not provide the necessary APIs, https://github.com/libressl-portable/portable/issues/381
....
```

* If Python is built successfully, you should see executable file `python` in the Python source directory. It's optional to run `make install`, instead you can run python directly at the source folder :
```
user@localhost: ~/xxx/$ ./python 
Python 3.12.0 (heads/3.12:0fb18b0, Oct 16 2023, 00:32:25) [GCC 10.3.0] on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> import datetime
>>> 
```

#### PIP
* Now install or upgrade `pip` for newly built python3.9 (you may need root priviledge to do this)
```
./python -m ensurepip --default-pip // first time to install
./python -m pip install whatever // force to reinstall

./python -m pip install --upgrade pip
```

* Recheck `pip` version by running `./python -m pip --version`, you should see result like this (pip version might be different in your case):
```
pip 23.2.1 from /usr/local/lib/python3.9/site-packages/pip (python 3.9)
```

* Tty install whatever packages you need using `pip` (example below), For those who built openssl from source, recheck the execution result, you should see download progress of the packages.  

```
./python -m pip install pytest
```



#### Reference
- https://devguide.python.org/setup/
- https://packaging.python.org/tutorials/installing-packages/
- https://bugs.python.org/issue34028
- https://www.guru99.com/pytest-tutorial.html
- https://stackoverflow.com/questions/51373063/pip3-bad-interpreter-no-such-file-or-directory
- https://stackoverflow.com/questions/53543477/building-python-3-7-1-ssl-module-failed

