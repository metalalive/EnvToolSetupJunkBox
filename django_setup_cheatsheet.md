Environment : Django version 3.1

#### Start development server
```
python manage.py  runserver --settings  <PATH.TO.SETTING_MODULE> --noreload  <YOUR_PORT_NAME>
```

* `<PATH.TO.SETTING_MODULE>` should be `settings.py` for your project or application
* `--noreload` avoids the running server from reloading itself whenever any file content related to your Django application / project is edited and saved.


#### Database Migration
Each time after you modify the models of an application, you must synchronize the update to low-level database schema, used by the appliction, by running `makemigrations` command :
```
python  manage.py  makemigrations  <YOUR_APP_NAME> --settings <PATH.TO.SETTING_MODULE>
```
then new migration file `<MIGRATION_SERIAL_NUMBER>_xxxx.py` is generated to `<YOUR_APP_NAME>/migrations`.

To further check the raw SQL commands generated for this migration, use `sqlmigrate` command :
```
python  manage.py  sqlmigrate  <YOUR_APP_NAME>  <MIGRATION_SERIAL_NUMBER> --database <DB_SETTING_KEY> \
    --settings <PATH.TO.SETTING_MODULE> --backwards
```
Note:
* `<DB_SETTING_KEY>` is the key value to specific database setup in `DATABASES` parameter of `settings.py`.
* `--backwards` is optional for viewing auto-generated SQL statements for migration rollback

Once you ensure everything in the migration file is what you expected, you commit the migration by running `migrate` command :
```
python  manage.py  migrate  <YOUR_APP_NAME>  <MIGRATION_NAME> --database <DB_SETTING_KEY> \
    --settings <PATH.TO.SETTING_MODULE>
```
* `<MIGRATION_NAME>` can be `<MIGRATION_SERIAL_NUMBER>` which is automatically generated sequence number by default by the command `django makemigration`, e.g. 0001, 0002 ...
* If `<MIGRATION_NAME>` is `zero`, that indicates django to revert all the way back to **the state before initial migration**. It also reset all migration records in `django_migration` database table (a table for internal use).


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

#### Test
```
python manage.py test  <YOUR_PACKAGE_PATH>.<YOUR_MODULE_NAME>   --settings  <PATH.TO.SETTING_MODULE> --keepdb
```
Note:
* `<YOUR_MODULE_NAME>` can be omitted, once omitted, Django will look for `__init__.py` at the path `<YOUR_PACKAGE_PATH>`, if the module is also not found, Django reports error and aborts the entire test.
* Use `--settings` to specify which application to test
* Django performs test operation with separate blank database with the name `test_<YOUR_DB_NAME_FOR_THE_PRODUCTION>` by default, it automatically creates that database only for testing purpose if it does NOT exist. Once created, `--keepdb` takes effect to keep the database schema after test finishes, so Django won't destroy the entire test database, If the model in your application and database schema don't change, then next time you can run test again without creating all required schemas again, which may save testing time.
* When running test, make sure the database user has enough access permission to perform the entire test. Since Django (as of v3.1) doesn't support to dynamically switch database user credential for testing, application developers will need to handle this by themselves.
* Also, the database user credential for testing requires full access privilege to the database `test_<YOUR_DB_NAME_FOR_PRODUCTION>` (if default name is applied).


#### Reference

* [django-admin and manage.py](https://docs.djangoproject.com/en/dev/ref/django-admin/)
* [Django testing](https://docs.djangoproject.com/en/dev/topics/testing/)
