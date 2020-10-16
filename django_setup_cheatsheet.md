Environment : Django version 3.1

#### Database Migration
Each time after you modify the models of an application, you must synchronize the update to low-level database schema, used by the appliction, by running `makemigrations` command :
```
python  manage.py  makemigrations  <YOUR_APP_NAME>
```

Then new migration file `<MIGRATION_SERIAL_NUMBER>_xxxx.py` is generated to `<YOUR_APP_NAME>/migrations`.
To further check the raw SQL commands generated for this migration, use `sqlmigrate` command :

```
python  manage.py  sqlmigrate  <YOUR_APP_NAME>  <MIGRATION_SERIAL_NUMBER>  | less
```

Once you ensure everything in the migration is what you expected, you commit the migration by running `migrate` command :
```
python  manage.py  migrate  <YOUR_APP_NAME>  <MIGRATION_SERIAL_NUMBER>
```

#### Accidental table deletion
If a database table, or column of a table for an application, is dropped accidentally , 
you should manually recover the tables in the SQL command line interface,
check out the SQL command(s) that built these tables by `sqlmigrate` :
```
python  manage.py sqlmigrate  YOUR_APP_NAME  <MIGRATION_SERIAL_NUMBER>
```

#### Dump records from database table to file

```
python manage.py dumpdata  --database <DB_SETTING_KEY> --format json --indent 4 --pks <PRIMARY_KEYS> \
    --output <PATH/TO/OUTPUT/FILE>  [<INSTALLED.APP.MODEL>  .....] 
```

where :
* `<DB_SETTING_KEY>` in `--database` option should be the key related to one of your working database settings.
* `<PRIMARY_KEYS>` in `--pks` option should be list of pk value seperated by comma `,` For example `--pks 4,8,9`
* `<INSTALLED.APP.MODEL>` can be any of installed applications you implemented specifically for your project, or installed applications in `django.contrib` , e.g.            `<INSTALLED.APP.MODEL>` can be `auth.Group` or `auth.User`
* If `--pks` option is in use, there must be ONLY one `<INSTALLED.APP.MODEL>` in the command, e.g.:

```
python3.9 manage.py dumpdata --database default --format json --indent 4 --pks 4,8 \
  --output my_fixtures/user_mgt/authrole.json  auth.Group
```

The structure of the output may look like this :

```
[
{
    "model": "auth.group",
    "pk": 4,
    "fields": {
        "name": "can manage geo location",
        "permissions": [ 26, 27, 28, 53 ]
    }
},
{
    "model": "auth.group",
    "pk": 8,
    "fields": {
        "name": "can manage email contact",
        "permissions": [ 29, 30, 31, 32,  57, 58, 59, 60 ]
    }
}
]
```


#### load well-formed data from file to database table

```
python manage.py loaddata  --database <DB_SETTING_KEY>  <PATH/TO/INPUT/FILE>
```

Note:
* `<PATH/TO/INPUT/FILE>` may be well-formed json or xml file (e.g. previously dumped from `manage.py dumpdata` command)
* if `FIXTURE_DIRS` is configured in `settings.py` , it will also look for `FIXTURE_DIRS/<PATH/TO/INPUT/FILE>`




#### Reference

* [django-admin and manage.py](https://docs.djangoproject.com/en/dev/ref/django-admin/)

