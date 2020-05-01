### Git useful commands 


#### shallow clone

carefully clone what you need (without cloning large amount of old history data)
```
cd NEW_FOLDER
git init
git remote add origin <GIT_REPO_URL>
git fetch --depth 1 origin <COMMIT_SHA1>
git checkout FETCH_HEAD
```

Alternative :
```
git clone --depth 1 <GIT_REPO_URL> --branch <BRANCH_NAME> 
```



