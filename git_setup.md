### Git useful commands 


#### shallow clone

carefully clone what you need (without cloning large amount of old history data)
```
cd NEW_FOLDER
git init
```

* Use [remote](https://git-scm.com/docs/git-remote) to specify which branch of remote repository your local codebase should synchronize with
```
git remote add  -t <BRANCH_NAME> <REMOTE_NAME> <GIT_REPO_URL>
```
e.g.
```
git remote add  origin https://my.github.codebases/project123
```

* To fetch codebase at specific commit without loading all older commits in the same branch. Note this command will generate [detached branch](https://stackoverflow.com/questions/10228760/how-do-i-fix-a-git-detached-head) :
```
git fetch --depth 1 <REMOTE_NAME>  <COMMIT_SHA1>
git checkout FETCH_HEAD
```


Alternative #1 , `--depth` works the same way, `HEAD` indicates that the git starts from the latest commit and download specific number of commits (in this case it is `3`):
```
git pull --depth 3 <REMOTE_NAME>  HEAD
```

Alternative #2 , it is not good for large-scale repositories (e.g. with millions of commits), because it may load unecessary old commits you've never used :
```
git clone --depth 1 <GIT_REPO_URL> --branch <BRANCH_NAME> 
```



