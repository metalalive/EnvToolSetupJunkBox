----------------------------------------------------------------------

### Useful commands for database management

##### start MariaDB server:
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  /PATH/TO/MARIADB/INSTALL/bin/mysqld_safe  --datadir='PATH/TO/DATABASE/FOLDER'  &
```

##### Shutdown MariaDB server
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  kill -SIGTERM <MARIADB_PID>
```

##### Login MariaDB through interactive command line interface :
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  /PATH/TO/MARIADB/INSTALL/bin/mysql -u  <USER_ACCOUNT> -p -h <IP_OR_DOMAIN_NAME>
```

##### List all existing databases
```
SHOW DATABASES;
```

##### switch to specific database
```
USE <YOUR_DATABASE_NAME>;
```

##### List all columns (and their attributes) of a database table
```
SHOW COLUMNS FROM <YOUR_TABLE_NAME>;
```


##### List attributes of all available database users
```
SELECT  host,user,max_connections,max_user_connections  FROM mysql.user
```

##### Check user privilege
Note don't list privilege fields of any user in `mysql.user`, they're NOT synchronized with `GRANT` command
```
SHOW GRANTS FOR  YOUR_USER_ACCOUNT@YOUR_HOSTNAME;
```

##### Grant privilege
Grant certain type(s) of privilege to specific database for specific user.
```
GRANT CREATE,DROP,INDEX, ANY_VALID_PRIVILEGE_OPTIONS  ON \
     `DATABASE_NAME`.* TO 'DB_USERNAME'@'IP_OR_DOMAIN_NAME';
```

It can also grant privileges to specific database table by modifying :
```
`DATABASE_NAME`.*
```
to 
```
`DATABASE_NAME`.`TABLE_NAME`
```

##### List table size of a specific database in descending order
```
SELECT table_name, ROUND(((data_length + index_length) / 1024), 2) `Size (KB)`\
    FROM information_schema.TABLES \
    WHERE TABLE_SCHEMA = 'YOUR_DATABASE_NAME' \
    ORDER BY (data_length + index_length)  DESC;
```

#### Show all tables you can drop in a database, but still keep database itself
```
SELECT CONCAT('DROP TABLE IF EXISTS `', TABLE_SCHEMA, '`.`', TABLE_NAME, '`;') \
FROM information_schema.TABLES WHERE TABLE_SCHEMA = 'mydb';
```
#### List all foreign key references to a database table
```
SELECT TABLE_NAME,COLUMN_NAME, REFERENCED_TABLE_NAME,REFERENCED_COLUMN_NAME \
     FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE \
     WHERE REFERENCED_TABLE_SCHEMA = 'da_name' \
     and REFERENCED_TABLE_NAME = 'table_name';
```


#### Create self-referencing table
```
CREATE TABLE `your_table` (`id` integer AUTO_INCREMENT NOT NULL PRIMARY KEY, `parent_id` integer NULL);
ALTER TABLE `your_table` ADD CONSTRAINT `your_contraint_name` FOREIGN KEY (`parent_id`) REFERENCES `your_table` (`id`);
```
Note that `NULL` has to be allowed in `parent_id`, otherwise you'll be in trouble when deleting rows in the table.


#### Delete all rows in a self-referencing table
```
UPDATE 'your_table' SET 'parent_id' = NULL WHERE 'parent_id' IS NOT NULL;
DELETE FROM 'your_table';
```


