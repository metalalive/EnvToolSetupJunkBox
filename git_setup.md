### Git useful commands 


#### shallow clone

carefully clone what you need (without cloning large amount of old history data)
```
cd NEW_FOLDER
git init
```

* Use [remote](https://git-scm.com/docs/git-remote) to specify which branch of remote repository your local codebase should synchronize with
```
git remote add  -t <BRANCH_NAME>  <ALIAS_NAME_OF_REMOTE_REPO>  <GIT_REPO_URL>
```
e.g.
```
git remote add -t my_remote_master  local_origin_123   https://my.github.codebases/project123
```

To fetch codebase at specific commit without loading all older commits in the same branch.
* Note this command will generate [detached branch](https://stackoverflow.com/questions/10228760/how-do-i-fix-a-git-detached-head)
* If you also want to switch between different tags, ensure you have enough commits in the `git fetch` command (that means, `--depth` should be large enough so you can switch to specific commit / tag later in `git checkout` command)
```
git fetch --depth 1 <REMOTE_NAME>  <COMMIT_SHA1> 
```

To switch between commits, you have [git-checkout](https://git-scm.com/docs/git-checkout).

- switch to the latest commit :
```
git checkout FETCH_HEAD
```
- switch to specific commit :
```
git checkout <SHA_HEXSTRING_OF_THE_COMMIT>
```
- switch to specific tag :
```
git checkout <TAG_NAME>
```
- switch to specific branch :
```
git checkout <EXISTING_BRANCH_NAME>
```

* List all tags which can be switched to in the local repository (depends on `--depth` in your previous `git fetch`)
```
git tag
```


* Alternative #1 , `--depth` works the same way, `HEAD` indicates that the git starts from the latest commit and download specific number of commits (in this case it is `3`):
```
git pull --depth 3 <REMOTE_NAME>  HEAD
```

* Alternative #2 , load all histories. It is not good for large-scale repositories (e.g. with millions of commits), because it may load unecessary old commits you've never used :
```
git clone --depth 1 <GIT_REPO_URL> --branch <BRANCH_NAME> 
```

To remove local commit without discarding the change :
* latest commit only
```
git reset HEAD^
```


