#### Environment :
* PostgreSQL v12.2
* Ubuntu 14.04 LTS
* OpenSSL 1.1.1c (built from source)


#### Build procedure
```
./configure  --prefix=/usr/local/pgsql  --enable-debug  --enable-profiling  --with-openssl \
    --with-libxml  CFLAGS='-I/usr/local/include -I/usr/include'  LDFLAGS='-L/usr/local/lib -L/usr/lib'
```

```
make COPT='-Werror'  >& build0.log &
```

```
make check >& regression_test.log &
```

```
make install >& install.log &
```


#### Reference
[PostgreSQL Install from source](https://wiki.postgresql.org/wiki/Compile_and_Install_from_source_code#Ubuntu_requirements)
