Environment : Django version 3.1


#### Accidental table deletion
If a database table, or column of a table for an application, is dropped accidentally , 
you should manually recover the tables in the SQL command line interface,
check out the SQL command(s) that built these tables by `sqlmigrate` :

```
python  manage.py sqlmigrate  YOUR_APP_NAME  0001
```

