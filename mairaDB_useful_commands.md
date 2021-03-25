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

##### Check version
```
SELECT VERSION();
```

##### List all existing databases
```
SHOW DATABASES;
```

##### List all storage engines available
```
SHOW ENGINES;
```
it will return the engine name, whether it supports transaction and savepoint

##### Show global / per-session variables
```
SHOW VARIABLES LIKE '%<KEYWORD>%';
SHOW SESSION VARIABLES LIKE '%<KEYWORD>%';
```
* it will show up current values applied to the global variables applied in the database server
* for each database connection, the per-session variables default to this global variables if not specified.

for example :
```
> SHOW VARIABLES LIKE '%auto%';
+------------------------------+-------+
| Variable_name                | Value |
+------------------------------+-------+
| auto_increment_increment     | 1     |
| auto_increment_offset        | 1     |
| autocommit                   | ON    |
| automatic_sp_privileges      | ON    |
| innodb_autoinc_lock_mode     | 1     |
| innodb_stats_auto_recalc     | ON    |
| sql_auto_is_null             | OFF   |
+------------------------------+-------+
> SHOW SESSION VARIABLES LIKE '%iso%';
+---------------+-----------------+
| Variable_name | Value           |
+---------------+-----------------+
| tx_isolation  | REPEATABLE-READ |
+---------------+-----------------+
```

##### switch to specific database
```
USE <YOUR_DATABASE_NAME>;
```

##### Show tables of a database
Print names of all table. Note that you have to run `USE <YOUR_DATABASE_NAME>` prior to this command
```
SHOW TABLES
```
Query table status with conditions, e.g.
```
SHOW TABLE STATUS FROM <YOUR_DATABASE_NAME> WHERE NAME LIKE '%<KEYWORD>%';
```
it will return more detail that shows metadata of the tables, and columns e.g. `Name`, `Engine`, `Version`, `Row_format`, `Rows`, `Avg_row_length`, `Data_length`, `Max_data_length`, `Index_length`, `Data_free`, `Auto_increment` `Create_time`,  `Update_time`, `Check_time`, `Collation`, `Create_options`, `Max_index_length`,  `Temporary`


##### List all columns (and their attributes) of a database table
```
SHOW COLUMNS FROM <YOUR_TABLE_NAME>;
```


##### List attributes of all available database users
```
SELECT  host, user, max_connections, max_user_connections  FROM mysql.user
SELECT Host, Db, User, Select_priv, Update_priv, grant_priv FROM mysql.db
```

##### Check user privilege
Note don't list privilege fields of any user in `mysql.user`, they're NOT synchronized with `GRANT` command
```
SHOW GRANTS FOR  YOUR_USER_ACCOUNT@YOUR_HOSTNAME;
```

##### Create new user
See [CREATE USER](https://mariadb.com/kb/en/create-user/)

##### [Grant privilege](https://mariadb.com/kb/en/grant/)
Grant certain type(s) of privilege to specific database for specific user.
```
GRANT  ANY_VALID_PRIVILEGE_OPTIONS  ON  `DATABASE_NAME`.`TABLE_NAME` TO 'DB_USERNAME'@'IP_OR_DOMAIN_NAME';
```
Note
* `ANY_VALID_PRIVILEGE_OPTIONS` can be a list of valid privilege options, they depend on [privilege level](https://mariadb.com/kb/en/grant/#privilege-levels)
* `TABLE_NAME` can also be wildcard character `*`, which means to grant the privileges to all tables under specific database :
  ```
  `DATABASE_NAME`.*
  ```
* **The command can also be completed even if `TABLE_NAME` points to non-existent database table.**

Example #1 : To modify max_user_connections of a DB user, you have :
```
GRANT USAGE ON `DATABASE_NAME`.* TO  'DB_USERNAME'@'IP_OR_DOMAIN_NAME'  WITH max_user_connections <NEW_VALUE>;
```

##### [Revoke privilege](https://mariadb.com/kb/en/revoke/)
Revoke certain type(s) of privileges that were granted to specific user.
```
REVOKE ANY_VALID_PRIVILEGE_OPTIONS  ON `DATABASE_NAME`.`TABLE_NAME`  FROM  'DB_USERNAME'@'IP_OR_DOMAIN_NAME';
```

##### [Query execution plan](https://mariadb.com/kb/en/explain/)
To make sure your query is optimal and makes use of existing index(es), you can check the execution plan by :
```
EXPLAIN <VALID_SQL_QUERY>;
```
* In mariadb `<VALID_SQL_QUERY>` has to be one of `SELECT`, `UPDATE`, `DELETE` statements.
* the output of execution plan looks like following result table, the important columns are :
  *  `key` column means whether to apply any key index (e.g. primary key, unique key, foreign key ... etc) when performing the given query at low-level storage engine implementation.
  *  `key_len` column means size of each entry of the applied index. For example, when an index named `PRIMARY KEY` is applied to this query execution and the primary-key column is a 6-byte character, then the `key_len` is 6.
  *  `rows` column means number of rows that will be generated in the result set
```
+------+-------------+----------------------------+-------+---------------+---------+---------+------+------+-------------+
| id   | select_type | table                      | type  | possible_keys | key     | key_len | ref  | rows | Extra       |
+------+-------------+----------------------------+-------+---------------+---------+---------+------+------+-------------+
|    1 | SIMPLE      | <YOUR_DB_TABLE_NAME>       | range | PRIMARY       | PRIMARY | 6       | NULL |  125 | Using where |
+------+-------------+----------------------------+-------+---------------+---------+---------+------+------+-------------+

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

#### Auto-increment key
##### [Reset auto-increment of a table](https://mariadb.com/kb/en/auto_increment-handling-in-innodb/#setting-auto_increment-values)
According to [this stackoverflow answer](https://stackoverflow.com/a/8923132/9853105) , If you use InnoDB as storage engine, you must ensure the reset value is greater than (not equal to) current maximum index (in the pk field of the table)

``` 
ALTER TABLE your_table_name AUTO_INCREMENT = <ANY_POSITIVE_INTEGER_VALUE>;
```
then double-check the value by running `SHOW CREATE TABLE <YOUR_TABLE_NAME>;` , you'll see the current `AUTO_INCREMENT` value of the table.

##### Assume Your table has auto-increment primary key, and it grows big (e.g. > 1m rows) ...
the default index (for that auto-increment pk) also grows , in some cases one insertion will lead to several splitting and restructuring operation on the index (due to the nature of B+tree), the restructure would then lead to page reallocation at low-level OS which spents more time on restructing.

[TODO: figure out how to solve it]


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

##### Specify collation in a query

Assume you attempts to retrieve the record in `<YOUR_TABLE>` (as shown below)

```
+----+----------+
| id | username |
+----+----------+
| 39 | JonSnow  |
+----+----------+
```

You will get nothing if specifying any collation that ends with `_bin` or `_cs`:

```
SELECT id, username FROM <YOUR_TABLE> WHERE username LIKE 'JoNsNoW' COLLATE utf8mb4_bin;
> Empty set (0.016 sec)
```

Or get the record by specifying case-insensitive collation:
```
SELECT id, username FROM <YOUR_TABLE> WHERE username LIKE 'JoNsNoW' COLLATE utf8mb4_unicode_ci;
> +----+----------+
> | id | username |
> +----+----------+
> | 39 | JonSnow  |
> +----+----------+
> 1 row in set (0.001 sec)
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



