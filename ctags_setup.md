#### Ctags setup

Create ctag lookup table for specific programming language
```
ctags --exclude=<FOLDER_OR_FILE_TO_EXCLUDE>  --languages=Python     -o py_ctags \
      -R <FOLDER_OR_FILE_LIST_TO_INCLUDE>
      
ctags --exclude=<FOLDER_OR_FILE_TO_EXCLUDE>  --languages=JavaScript -o js_ctags \
      -R <FOLDER_OR_FILE_LIST_TO_INCLUDE>
```

where :
* `FOLDER_OR_FILE_TO_EXCLUDE` : e.g. `.git`
* `FOLDER_OR_FILE_LIST_TO_INCLUDE` : e.g. `src` `include` `folder1` `folder2` `setup.py` `startconfig.py` 


Import the lookup tag file to Vim editor. For example :

```
:set tags=./py_ctags;
```
