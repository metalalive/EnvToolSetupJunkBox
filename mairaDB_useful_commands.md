----------------------------------------------------------------------

### Useful commands for database management

##### start MariaDB server:
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  /PATH/TO/MARIADB/INSTALL/bin/mysqld_safe  --datadir='PATH/TO/DATABASE/FOLDER'  &
```
or launch DB server with configuration file in which you can set all valid variables :
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  /PATH/TO/MARIADB/INSTALL/bin/mysqld_safe  --defaults-file='PATH/TO/CONFIG_FILE'  &
```

##### Shutdown MariaDB server
```
sudo -u <OS_ACCOUNT_FOR_MARIADB>  kill -SIGTERM  <mysqld_PID>
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
Example #1 : To modify max_user_connections of a DB user, you have :
```
GRANT USAGE ON `DATABASE_NAME`.* TO  'DB_USERNAME'@'IP_OR_DOMAIN_NAME'  WITH max_user_connections <NEW_VALUE>;
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

#### Reset auto-increment of a table
According to [this stackoverflow answer](https://stackoverflow.com/a/8923132/9853105) , If you use InnoDB as storage engine, you must ensure the reset value is greater than (not equal to) current maximum index (in the pk field of the table)

``` 
ALTER TABLE your_table_name AUTO_INCREMENT = <ANY_POSITIVE_INTEGER_VALUE>;
```

#### Character set and  Collation in database or table 

Quick reminder:
* One character set can have one default collation, and more than one collations, while one collation cannot be in more than one different character sets.
* Each time user could sends a query with specific collation in the character defined in table schema.
* User can specify character set and default collation at database level, table level, or column level.
* Conventionally collations naming starts with character set name, ends with case sensitivity (e.g. `_ci` means case insensitive, `_cs` means case sensitive). However case-sensitive collations (`_cs`) may not be provided, in such case, you could use `_bin` instead:

> e.g. in MariaDB, character set `utf8` has collation like `utf8_unicode_ci` , but doesn't have case-sensitive version like `utf8_unicode_cs`, so you should try using `utf8_bin` as substitution

*  Character set `utf8` is deprecated, it is recommended to use `utf8mb4` instead.
 
##### Check out character sets and collations

To view all available collations of certain charset (e.g. utf8), you have:
```
SHOw COLLATION LIKE 'utf8_%';
```
or all collations which end with `_cs`:
```
SHOw COLLATION LIKE '%_cs';
```

To view default collation of a database table (then see the `collation` column)
```
SHOW TABLE STATUS WHERE name LIKE '<YOUR_TABLE_NAME>';
```

To view default character set and collation of all databases
```
SELECT schema_name, default_character_set_name, default_collation_name FROM information_schema.SCHEMATA;
+--------------------+----------------------------+------------------------+
| schema_name        | default_character_set_name | default_collation_name |
+--------------------+----------------------------+------------------------+
| performance_schema | utf8                       | utf8_general_ci        |
| mysql              | latin1                     | latin1_swedish_ci      |
| information_schema | utf8                       | utf8_general_ci        |
| <YOUR_DATABASE>    | utf8mb4                    | utf8mb4_bin            |
+--------------------+----------------------------+------------------------+
```

##### Change character sets and collations

For example, convert to charset `utf8` and default collation `utf8mb4_bin` 

At database level:
```
ALTER DATABASE <YOUR_DATABASE_NAME> CHARACTER SET = 'utf8mb4' COLLATE = 'utf8mb4_bin';
> Query OK, 1 row affected (0.042 sec)
```

At table level
```
ALTER TABLE <YOUR_DATABASE_TABLE_NAME> CONVERT TO CHARACTER SET 'utf8mb4' COLLATE 'utf8mb4_bin';
> Query OK, 7 rows affected (2.408 sec)
> Records: 7  Duplicates: 0  Warnings: 0
```


#### Prepare Configuratio File

Example settings below, it is good practice to turn on `general_log` ONLY for debugging purpose since it prints EVERY query user executed
```
[mariadb]
datadir=./data
log_error=localhost.err
log_warnings=9
general_log=1
general_log_file=localhost1234.log
```

Some variables can also be turned on/off at runtime (after database server started) using `SET GLOBAL` command in mysql CLI.
Note some variables are read-only cannot be modified (e.g. `log_error`)
```
> SET GLOBAL <WRITEABLE_VARIABLE_NAME> = <NEW_VALUE>;
> SHOW GLOBAL VARIABLES LIKE <VARIABLE_NAME>;
```



### Reference
* [Max_used_connections per user/account](https://www.fromdual.com/max-used-connections-per-user-account)
* [All Supported System Variables](https://mariadb.com/kb/en/replication-and-binary-log-system-variables/)
* [Configuration Files](https://mariadb.com/kb/en/configuring-mariadb-with-option-files/)
* [MariaDB logs](https://mariadb.com/kb/en/overview-of-mariadb-logs/)
* [MariaDB: Setting Character Sets and Collations](https://mariadb.com/kb/en/setting-character-sets-and-collations/)
* [Difference between utf8_general_ci and utf8_unicode_ci?](https://stackoverflow.com/questions/766809/)
* [How to set the encoding for the tables' char columns in django?](https://stackoverflow.com/questions/1198486/)
* [How do I see what character set a MySQL database / table / column is?](https://stackoverflow.com/q/1049728/9853105)
* [Case sensitive search in Django, but ignored in Mysql](https://stackoverflow.com/q/28073941/9853105)



