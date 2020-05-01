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

After installation, ensure you have seperate account (e.g. user account `postgresql`) in your OS which has full access permissions to all these binary files / library files. It is for security.


--------------------------------------------------------------------

#### Useful commands for database management

**NOTE** you may need to run these commands below with the dedicate OS user account as mentioned above, e.g. in Ubuntu it would be `sudo -u YOUR_DEDICATE_PGSQL_USER  SUPPORTED_PGSQL_CMD`



##### start database server

You must start database server first prior to all other database commands described below
```
./bin/postgres -D PATH/TO/DB/FILES -h YOUR_DB_HOST_DOMAIN_NAME  &> pgsrv.log &
```

##### stop database server
```
./bin/pg_ctl  stop   -D  PATH/TO/DB/FILES -m SHUTDOWN_MODE
```
where `SHUTDOWN_MODE` can be one of `smart`, `fast`, `immediate`. (the last 2 options may cause data loss ?)



##### Launch interactive command line interface 

with specified database name & its owner
```
./bi/psql  --host=YOUR_DB_HOST_DOMAIN_NAME  --dbname=YOUR_DB_NAME
```

##### create new DB user
```
./bin/createuser  --connection-limit=2  --superuser  --login  --host=YOUR_DB_HOST_DOMAIN_NAME \
    --password  DB_USER_NAME
```
* `--superuser` is optional, you can modify access permission to the user account later using SQL command `ALTER`.
* `--password` will prompt user to enter the password
* `YOUR_DB_HOST_DOMAIN_NAME`, e.g. `localhost` or `127.0.0.1`

##### drop DB user
```
./bin/dropuser  odoodba1
```

##### create new database
```
./bin/createdb  --owner=DB_OWNER_USER_NAME  --host=YOUR_DB_HOST_DOMAIN_NAME \
    --username=CONNECT_DB_USER_NAME  --password
```

* `DB_OWNER_USER_NAME` may not have permission to drop entire database
* `CONNECT_DB_USER_NAME` means the user account to connect to database then perform this `createdb` operation


##### List databases available

* `./bin/psql -l`
* `./bin/psql  --host=YOUR_DB_HOST_DOMAIN_NAME  --dbname=YOUR_DB_NAME --command="\l" `


##### List all existing tables  of a specified database
```
./bin/psql  --host=YOUR_DB_HOST_DOMAIN_NAME  --dbname=YOUR_DB_NAME --command="\d"
```


##### List attributes of a specified table

e.g. names, data type of each columns, index, primary key, foreign key references ...etc.
```
./bin/psql  --host=YOUR_DB_HOST_DOMAIN_NAME  --dbname=YOUR_DB_NAME --command="\d YOUR_TABLE_NAME"
```

##### List currently established database connections
```
./bin/psql --host=YOUR_DB_HOST_DOMAIN_NAME  --dbname=YOUR_DB_NAME --username=DB_USER_NAME \
    --command="SELECT datname, pid, usename, state, wait_event,  query FROM pg_stat_activity;"
```

##### Drop all tables and their data records of the same database schema
```
./bin/psql  --host=YOUR_DB_HOST_DOMAIN_NAME  --dbname=YOUR_DB_NAME \
    --command="DROP SCHEMA YOUR_SCHEMA_NAME CASCADE; CREATE SCHEMA YOUR_SCHEMA_NAME;"
```

In most cases, `YOUR_SCHEMA_NAME` is `public`.



#### Reference
[PostgreSQL Install from source](https://wiki.postgresql.org/wiki/Compile_and_Install_from_source_code#Ubuntu_requirements)
