### Git useful commands 


#### Shallow Clone
Carefully clone what you need (without cloning large amount of old history data).

In GIT, you can determine number of commits to download from remote repo.

```
cd NEW_FOLDER
git init
```

* Use [remote](https://git-scm.com/docs/git-remote) to specify which branch of remote repository your local codebase should synchronize with
```
git remote add  -t <REMOTE_BRANCH_NAME>  <ALIAS_NAME>  <GIT_REPO_URL>
```
where
- `<REMOTE_BRANCH_NAME>` : branch name created at remote repo
- `<ALIAS_NAME>` : alias name of the remote git repo, referenced at local git repo

e.g.
```
git remote add -t my_remote_master  local_origin_123   https://my.github.codebases/project123
```

To fetch codebase at specific commit without loading all older commits in the same branch.
* Note this command will generate [detached branch](https://stackoverflow.com/questions/10228760/how-do-i-fix-a-git-detached-head)
* If you also want to switch between different tags, ensure you have enough commits in the `git fetch` command (that means, `--depth` should be large enough so you can switch to specific commit / tag later in `git checkout` command)
```
git fetch --depth  <NUM_COMMITS>  <ALIAS_NAME>  <COMMIT_SHA1> 
```
where
- `<NUM_COMMITS>` indicates number of consecutive commits to fetch, starting from the commit represented as `<COMMIT_SHA1>`, going back to the nth. old commit
- `<ALIAS_NAME>`: name of the remote git repo, previously created on `git remote add`

#### Branch
List existing branches of current repo
```
git branch
```
Create a new (local) branch under current local repo
```
git branch <LOCAL_LABEL>
```
Note:
- local branches may be different from remote branches in the same repo
- For [Shallow Clone](#shallow-clone) where a default [detached branch](https://stackoverflow.com/questions/10228760/how-do-i-fix-a-git-detached-head) is created, it is a good practice to create other local branches (e.g. one for backup, others for new patches) before you move on and change the codebase.
- Once you switch from detached branch to any other branch (by `git checkout <ANY_LOCAL_BRANCH>`), the detached branch will disappear, it is better NOT to change the code under detached branch.

#### Switch between commits
Consider `commit` as single node of an arbitrary tree structure, you can switch between them back and forth using [git-checkout](https://git-scm.com/docs/git-checkout).
```
git checkout <COMMIT_LABEL>
```
where the `<COMMIT_LABEL>` could be one of followings:
|syntax|meaning|
|------|-------|
|`FETCH_HEAD`|to the latest commit of current branch|
|`<SHA1_HEX_STRING>`|SHA1 string, represent the specific commit|
|`<TAG_NAME>`|to specific tag|
|`<EXISTING_LOCAL_BRANCH>`|to specific local branch|

#### Update change to remote repo
With the command [git-push](https://git-scm.com/docs/git-push)

Preresuisites : 
- switch to the appropriate [local branch](#branch) you want to update (source)
- specify appropriate remote branch (destination)
```
git checkout <EXISTING_LOCAL_BRANCH>
git push -u <ALIAS_NAME>   HEAD:<EXISTING_REMOTE_BRANCH>
```

#### Misc
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


