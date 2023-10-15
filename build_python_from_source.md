
### Python 3.12
Tested on Ubuntu 14.04LTS & Debain 9 (Raspbian Stretch)

* Install dependency packages
```
sudo apt-get install -y build-essential tk-dev libncurses5-dev libncursesw5-dev libreadline6-dev \
             libdb5.3-dev libgdbm-dev libsqlite3-dev libssl-dev libbz2-dev libexpat1-dev \
             liblzma-dev zlib1g-dev libffi-dev tar wget vim
```

* Download [Python source code](https://github.com/python/cpython) & Go to the downloaded Python source directory. **Please avoid ALPHA release or BETA release , always use FINAL release instead, you will save a lot of time**

* Configure everything required (you may need root priviledge to do things). For those who built OpenSSL from source for whatever reasons, you better specify path of your openssl installation path, with `--with-openssl`, `CFLAGS` and `LDFLAGS` options when running `./configure` below:

```
./configure  --with-openssl=/PATH/TO/YOUR/OPENSSL_INSTALL_FODLER/ \
    --enable-optimizations \
    --with-ssl-default-suites=openssl \
    CFLAGS="-I/PATH/TO/YOUR/OPENSSL_INSTALL_FODLER/include" \
    LDFLAGS="-L/PATH/TO/YOUR/OPENSSL_INSTALL_FODLER/"
```

* Check out `config.log`. For those who built OpenSSL from source, make sure you get following results :
```
configure:xxxxx: checking for openssl/ssl.h in /PATH/TO/YOUR/OPENSSL_INSTALL_FODLER/
configure:xxxxx: result: yes
configure:xxxxx: checking whether compiling and linking against OpenSSL works
configure:xxxxx: result: yes
configure:xxxxx: checking for X509_VERIFY_PARAM_set1_host in libssl
configure:xxxxx: result: yes
```
for newer version, the log may look like this :
```
checking for openssl/ssl.h in /PATH/TO/YOUR/OPENSSL_INSTALL_FODLER/ ... yes
checking whether compiling and linking against OpenSSL works... yes
checking for X509_VERIFY_PARAM_set1_host in libssl... yes
checking for --with-ssl-default-suites... openssl
```

> In 3.12, the configuration script starts checking stdlib extension module, it would complain that some modules are missing, such as `_ssl` or `_hashlib`, strangely , the build process later can be completed without the affect , and later `pip` installation works without TLS or security issues.

* start building
```
make build_all -j 1 >& build.log
```

* Recheck build.log. For those who built OpenSSL from source, make sure you DON'T have the following message in your `build.log`, otherwise you will get some troubles later when you try to install packages through `pip`.
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

https://devguide.python.org/setup/

https://packaging.python.org/tutorials/installing-packages/

https://bugs.python.org/issue34028

https://www.guru99.com/pytest-tutorial.html

https://stackoverflow.com/questions/51373063/pip3-bad-interpreter-no-such-file-or-directory

https://stackoverflow.com/questions/53543477/building-python-3-7-1-ssl-module-failed

