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

