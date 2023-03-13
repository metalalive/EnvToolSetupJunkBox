----------------------------------------------------------------------

## Useful commands for database management

### Basic

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

##### [Show status](https://mariadb.com/kb/en/show-status/)
provide  status information on an active server
```
SHOW STATUS WHERE variable_name = '<VALID_VARIABLE_NAME>';
```

for exmeple, to check number of running threads (created connections) in the database server , you have :
```
SHOW STATUS WHERE variable_name = 'threads_connected';

+-------------------+-------+
| Variable_name     | Value |
+-------------------+-------+
| Threads_connected | 13    |
+-------------------+-------+
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
it will return more detail that shows metadata of the tables, and columns e.g. `Name`, `Engine`, `Version`, `Row_format`, `Rows`, `Avg_row_length`, `Data_length`, `Max_data_length`, `Index_length`, `Data_free`, `Auto_increment` `Create_time`,  `Update_time`, `Check_time`, `Collation`, `Create_options`, `Max_index_length`,  `Temporary`.

Following statement shows table schema in similay way:
```
SHOW CREATE TABLE <YOUR_TABLE_NAME>;
```

##### Delete and ignore
When mariadb database encounters any error on `DELETE` operation , it aborts the process of deleting rows even the subsequent rows (expected to be deleted) are safe to delete (so left them unprocessed).
In such situation you could try `INGORE` on `DELETE` operation to ignore the ignorable errors, and switch to next row to delete  
```
DELETE IGNORE FROM <YOUR_TABLE_NAME> WHERE <ANY_VALID_CONDITION_CLAUSE>;
```
**BUT** be aware of [out-of-sync issue if you have master/slave mysql replica](https://www.percona.com/blog/2012/02/02/stop-delete-ignore-on-tables-with-foreign-keys-can-break-replication/)  in your deployment.


### Privilege
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

Quick example:
```
CREATE USER '<NEW_DB_ACCOUNT>'@'<NEW_DB_HOSTNAME>' IDENTIFIED BY '<YOUR_PASSWORD_PLAIN_TEXT>' \
    WITH  MAX_USER_CONNECTIONS 5  MAX_QUERIES_PER_HOUR 7200;
```

##### Rename existing user
```
RENAME USER  '<OLD_DB_ACCOUNT>'@'<OLD_DB_HOSTNAME>' TO  '<NEW_DB_ACCOUNT>'@'<NEW_DB_HOSTNAME>';
```

##### modify user attributes
Use [ALTER USER](https://mariadb.com/kb/en/alter-user/). Quick example:
```
ALTER USER  '<YOUR_DB_ACCOUNT>'@'<YOUR_DB_HOSTNAME>' IDENTIFIED BY '<NEW_PASSWORD_PLAIN_TEXT>' \
    WITH MAX_CONNECTIONS_PER_HOUR 100 ;
```


##### [Grant privilege](https://mariadb.com/kb/en/grant/)
Grant certain type(s) of privilege to specific database for specific user.
```
GRANT  ANY_VALID_PRIVILEGE_OPTIONS  ON  `DATABASE_NAME`.`TABLE_NAME` TO 'DB_USERNAME'@'IP_OR_DOMAIN_NAME';
```
For [column privilege](https://mariadb.com/kb/en/grant/#column-privileges), you need to add a list of column names wrapped by parenthesis between `SELECT` and `ON` keywords :
```
GRANT SELECT (`column_name_1`, `column_name_2`, `column_name_3`) ON `DATABASE_NAME`.`TABLE_NAME` TO 'DB_USERNAME'@'IP_OR_DOMAIN_NAME';
```

Note
* `ANY_VALID_PRIVILEGE_OPTIONS` can be a list of valid privilege options, they depend on [privilege level](https://mariadb.com/kb/en/grant/#privilege-levels) , such as `INSERT`, `DELETE`, `CREATE`, `DROP`, `GRANT OPTION` ... etc
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
```sql
REVOKE ANY_VALID_PRIVILEGE_OPTIONS  ON `DATABASE_NAME`.`TABLE_NAME`  FROM  'DB_USERNAME'@'IP_OR_DOMAIN_NAME';
```
Note:
- To completely remove specific user from accessing specific database, `ANY_VALID_PRIVILEGE_OPTIONS` has to include different levels of privileges :
  - [column privilege](https://mariadb.com/kb/en/grant/#column-privileges), such as `INSERT`, `SELECT`
  - [table privilege](https://mariadb.com/kb/en/grant/#column-privileges), such as `CREATE TABLE`, `ALTER TABLE`, `DROP TABLE`, more importantly, `GRANT OPTION`
- For example
  ```sql
  REVOKE  UPDATE, ALTER, GRANT OPTION  ON `DATABASE_NAME`.`TABLE_NAME`  FROM  'DB_USERNAME'@'IP_OR_DOMAIN_NAME';
  ```

-------------------------------------------------------------------------------------------

### Index
#### Show all indexes of a table
```
SHOW INDEX FROM <YOUR_TABLE_NAME>;
```
#### Compound primary key in a table
```
CREATE TABLE <YOUR_TABLE_NAME> (t1id INT, t2id INT, PRIMARY KEY (t1id, t2id));
```
The SQL statement above also works in **postgre SQL** and **Oracle**.
By checking the table schema, you should see both of `t1id` and `t2id` are marked as primary key.
```
> SHOW COLUMNS FROM <YOUR_TABLE_NAME>;
+------------+------------------+------+-----+---------+-------+
| Field      | Type             | Null | Key | Default | Extra |
+------------+------------------+------+-----+---------+-------+
| t1id       | int(4) unsigned  | NO   | PRI | NULL    |       |
| t2id       | int(4) unsigned  | NO   | PRI | NULL    |       |
+------------+------------------+------+-----+---------+-------+
```
The result set of the SQL statement above would be :
```
> SHOW INDEX FROM <YOUR_TABLE_NAME>;
-+------------------+------------+------------+--------------+-------------+-----------+-------------+----------+--------+------+------------+---------+
| Table             | Non_unique | Key_name   | Seq_in_index | Column_name | Collation | Cardinality | Sub_part | Packed | Null | Index_type | Comment |
+-------------------+------------+------------+--------------+-------------+-----------+-------------+----------+--------+------+------------+---------+
| <YOUR_TABLE_NAME> |          0 | PRIMARY    |            1 | t1id        | A         |           0 |     NULL | NULL   |      | BTREE      |         |
| <YOUR_TABLE_NAME> |          0 | PRIMARY    |            2 | t2id        | A         |           0 |     NULL | NULL   |      | BTREE      |         |
+-------------------+------------+------------+--------------+-------------+-----------+-------------+----------+--------+------+------------+---------+
```
In the example above :
* `key_name` indicates a valid index `<VALID_INDEX_NAME>`, in `<YOUR_TABLE_NAME>` the index name is `PRIMARY` which is used by the PRIMARY KEY indexing and consists of 2 columns : `t1id` and `t2id`.
* `Seq_in_index` indicates the ordering of the columns `t1id` and `t2id` in the [multiple-part index](https://dev.mysql.com/doc/refman/5.7/en/range-optimization.html#range-access-multi-part)  named `PRIMARY`, that shows how InnoDB organizes the compound key in its internal storage structure (typically B-tree). This provides advantages when you can only provide part of key column(s) (not all of them, e.g. only provide `t1id` in WHERE clause) in the SQL statement, so each record can be found and still using the `PRIMARY` index  (without full table scan). 
* each entry of the index contains 8-byte data, upper 4-byte part is used to store for one column of the compound key (either `t1id` or `t2id`), lower 4-byte part is used to store for the other column.
* One table may have more than one indexes, e.g. extra index for unique constraint, index for each foreign-key column (MySQL and MariaDB actually do so by default).
* Number of indexes within a table has tradeoff, more indexes might (or might not, sometimes) speed up read operations, but slow down write operations  (especially insertions) because your database needs to maintain all existing indexes of the table on the single write.

#### List all constraints from other tables referneced to a given table
```
SELECT table_name, column_name, constraint_name, referenced_table_name, referenced_column_name  \
    FROM `information_schema`.`key_column_usage`  WHERE  referenced_table_schema  = '<YOUR_DATABASE_NAME>' \
    AND referenced_table_name = '<YOUR_REFERENCED_TABLE_NAME>';
```
where `constraint_name` can be used as `<VALID_INDEX_NAME>` in `DROP INDEX` command (as shown below)


#### Add Primary Key Index
To add  new primary key to existing table (auto-increment attribute is optional) :
```SQL
ALTER TABLE  `<YOUR_TABLE_NAME>` ADD COLUMN `<NEW_COLUMN_NAME>` int NOT NULL AUTO_INCREMENT PRIMARY KEY
```

For BLOB/TEXT column, you have to specify first N bytes/chars to be in the primary key index :
```SQL
ALTER TABLE `<YOUR_TABLE_NAME>` ADD PRIMARY KEY (<VALID_COLUMN_NAME>(<YOUR_FIRST_N_BYTES>))
```

#### Drop index
In most cases you can simply delete an index without any error
```
ALTER TABLE `<YOUR_TABLE_NAME>` DROP INDEX `<VALID_INDEX_NAME>`
DROP INDEX `<VALID_INDEX_NAME>` ON `<YOUR_TABLE_NAME>`;
```
Note :
* the `<VALID_INDEX_NAME>` can be retrieved by `SHOW INDEX` command above or [listing index data](#list-index-data)
* `<VALID_INDEX_NAME>` can also be ``PRIMARY`` in order to remove PRIMARY KEY index
* [`DROP INDEX`](https://mariadb.com/kb/en/drop-index/) is mapped to [`ALTER TABLE ... DROP INDEX ...`](https://mariadb.com/kb/en/alter-table/).

For primary key, The alternative to remove `PRIMARY KEY index` in mariadb is :
 ```SQL
 ALTER TABLE <YOUR_TABLE_NAME> DROP PRIMARY KEY
 ```

If the primary key is auto-increment column, you will get error like :
```
ERROR 1075 (42000): Incorrect table definition; there can be only one auto column and it must be defined as a key
```
In this case, you should first drop the primary key **without** removing the column :
```SQL
 ALTER TABLE <YOUR_TABLE_NAME> MODIFY COLUMN `id` int NOT NULL, DROP PRIMARY KEY;
```
assume the `id` column has auto-increment property, you have to remove the primary key first , then drop the column `id` using another SQL statement :
```SQL
 ALTER TABLE <YOUR_TABLE_NAME> DROP COLUMN id;
```
 The 2 statements above cannot be combined into one, which is strange.



#### Drop compound-key index which includes foreign-key constraint
In some cases the compound-key index may require to reference a foreign key constraint, however, **mysql / mariadb seems to internally do some magic and let the foreign key pointed to the index** , so you will get the following error on `DROP INDEX` :
```
ERROR 1553 (HY000): Cannot drop index '<VALID_INDEX_NAME>': needed in a foreign key constraint
```

You have 2 options as workaround :
* temporarily delete the foreign key constraint, immediately drop the compound index, then create the same foreign key constraint back
  ```
  ALTER TABLE `<YOUR_TABLE_NAME>` DROP FOREIGN KEY `<YOUR_FK_CONSTRINT_NAME>`;
  ALTER TABLE `<YOUR_TABLE_NAME>` DROP INDEX <YOUR_INDEX_NAME>;
  ALTER TABLE `<YOUR_TABLE_NAME>` ADD CONSTRAINT `<YOUR_FK_CONSTRINT_NAME>` FOREIGN KEY (`<FK_IN_YOUR_TABLE>`) REFERENCES `<YOUR_REFERENCED_TABLE>`(`<YOUR_REFERENCED_COLUMN>`)
  ```
  * the downside of this option is : the constraint name `<YOUR_FK_CONSTRINT_NAME>` includes hashed value, you need to fetch the correct constraint name (by `SHOW CREATE TABLE <YOUR_TABLE_NAME>`) before you can delete the foreign key constraint...
* temporarily disable the key constraints of the table, drop the compound index, then enable them back later.
See [here](https://stackoverflow.com/questions/63497147) for issue description
  ```
  ALTER TABLE `<YOUR_TABLE_NAME>` DISABLE KEYS;
  ALTER TABLE `<YOUR_TABLE_NAME>` DROP INDEX `<YOUR_INDEX_NAME>`;
  ALTER TABLE `<YOUR_TABLE_NAME>` ENABLE KEYS;
  ```
  
*(TODO: what if we want to adjust the columns applied to the compound PRIMARY KEY ?)*



### Statistic data
#### [Query execution plan](https://mariadb.com/kb/en/explain/)
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

#### List table size of a specific database in descending order
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

#### List index data
To get metadata of an index from a InnoDB-based table, you can perform query on `mysql.innodb_index_stats`. For example, to get index size of a specific table, you have :
```
SELECT table_name, index_name, stat_name, stat_value, ROUND(stat_value * @@innodb_page_size / 1024, 2)\
    size_in_kb FROM mysql.innodb_index_stats WHERE database_name = '<YOUR_DB_NAME>' AND \
    table_name LIKE '%<YOUR_KEYWORD>%';
```
Note :
* `@@innodb_page_size` means to read value from global variable  `innodb_page_size`, which defaults to 16 KB
* `stat_name` could be `size` (total number of pages used in the entire index, remind that innodb use B-tree+ as index data structure) , or `n_leaf_pages` (total number of leaf pages for the index)

#### Pages allocated in index(es)
`information_schema.innodb_buffer_page` table contains information about pages in the buffer pool. For example, to get information of the pages allocated to indexes of a table, you have :
```
SELECT INDEX_NAME, PAGE_NUMBER, NUMBER_RECORDS, DATA_SIZE, PAGE_STATE, PAGE_TYPE  FROM  information_schema.innodb_buffer_page WHERE TABLE_NAME = '`<YOUR_DB_NAME>`.`<YOUR_TABLE_NAME>`' ORDER BY PAGE_NUMBER ;
```
* `NUMBER_RECORDS` The number of records within the page.
* `DATA_SIZE` The sum of the sizes of the records. This column is applicable only to pages with a `PAGE_TYPE` value of `INDEX`. 

#### Show [process list](https://mariadb.com/kb/en/show-processlist/)
This command shows you which threads are running, can be used to check connection issues. You can also get the information from [`information_schema.PROCESSLIST` table](https://mariadb.com/kb/en/information-schema-processlist-table/)

```
SHOW PROCESSLIST ;
MariaDB [Restaurant]> show processlist ;
+-------+-----------------+-----------------+------------+---------+------+--------------------------+------------------+----------+
| Id    | User            | Host            | db         | Command | Time | State                    | Info             | Progress |
+-------+-----------------+-----------------+------------+---------+------+--------------------------+------------------+----------+
|     2 | system user     |                 | NULL       | Daemon  | NULL | InnoDB purge worker      | NULL             |    0.000 |
|     1 | system user     |                 | NULL       | Daemon  | NULL | InnoDB purge coordinator | NULL             |    0.000 |
| 10567 | root            | localhost       | Restaurant | Query   |    0 | Init                     | show processlist |    0.000 |
| 10826 | app_admin       | localhost:40428 | Restaurant | Sleep   | 1630 |                          | NULL             |    0.000 |
+-------+-----------------+-----------------+------------+---------+------+--------------------------+------------------+----------+

```


### Misc.
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

#### List gap values associated with a column (mostly primary key) of a table
The following SQL prints the ranges of the gap values that haven't been used in the column :
```
SELECT  m3.lowerbound + 1, m3.upperbound - 1 FROM ( \
    SELECT m1.id as lowerbound, MIN(m2.id) as upperbound FROM deadlock_test m1 \
    INNER JOIN deadlock_test AS m2 ON m1.id < m2.id  GROUP BY m1.id \
    ) m3  WHERE m3.lowerbound < m3.upperbound - 1;
```
If the column is key (primary key, foreign key ... etc) in the subquery , you should see the key index is applied in the execution plan :
```
EXPLAIN SELECT m1.id as lowerbound, MIN(m2.id) as upperbound FROM deadlock_test m1 INNER JOIN deadlock_test AS m2 ON m1.id < m2.id  GROUP BY m1.id;
+------+-------------+-------+-------+---------------+---------+---------+------+------+--------------------------------------------------------------+
| id   | select_type | table | type  | possible_keys | key     | key_len | ref  | rows | Extra                                                        |
+------+-------------+-------+-------+---------------+---------+---------+------+------+--------------------------------------------------------------+
|    1 | SIMPLE      | m1    | index | PRIMARY       | PRIMARY | 4       | NULL |   11 | Using index; Using temporary; Using filesort                 |
|    1 | SIMPLE      | m2    | index | PRIMARY       | PRIMARY | 4       | NULL |   11 | Using where; Using index; Using join buffer (flat, BNL join) |
+------+-------------+-------+-------+---------------+---------+---------+------+------+--------------------------------------------------------------+
```
For detail, please read [here](https://stackoverflow.com/a/66669932/9853105) 


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

#### Deadlock in MySQL / MariaDB
##### Case 1 : Lock non-existent rows on concurrent insertion
This is an [old bug in MySQL community](https://bugs.mysql.com/bug.php?id=25847) that hasn't been fixed since 2007, For more discussion please read [this](https://stackoverflow.com/a/58526440/9853105) .
You can reproduce that by acquiring gap lock (also called [insert intention lock](https://dev.mysql.com/doc/refman/8.0/en/innodb-locking.html#innodb-insert-intention-locks) in MySQL / InnoDB), performing 2 insertions to the same table within 2 separate sessions , the execution flow is described below :
```
Session #1: CREATE TABLE mytable (id integer PRIMARY KEY) ENGINE=InnoDB;
Session #1: SET AUTOCOMMIT = 0;
Session #2: SET AUTOCOMMIT = 0;
Session #1: SELECT id FROM mytable WHERE id = 1 FOR UPDATE; -- succeed
Session #2: SELECT id FROM mytable WHERE id = 2 FOR UPDATE; -- succeed
Session #1: INSERT INTO mytable (id) VALUES (1); -- Hang
Session #2: INSERT INTO mytable (id) VALUES (2); -- Session #1: OK, Session #2: Deadlock found when trying to get lock; try restarting transaction
```
To avoid this issue on the concurrent writes, the alternative is to use `START TRANSACTION; <VALID_SQL_STATEMENTS>;  COMMIT;` instead of relying on `SELECT ... FOR UPDATE`. For example :
```
START TRANSACTION;
SELECT id FROM mytable WHERE id = 2 ;
INSERT INTO mytable (id) VALUES (2);
COMMIT;
```
So it will NOT cause deadlock even when 2 sessions insert a row concurrently to the same table

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



