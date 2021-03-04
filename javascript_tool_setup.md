Environment: Ubuntu, Debian 9


#### Node.js
If the distro-stable version is outdated, you may install newer version by following the steps :

* remove old version of node.js
* add new key for nodejs 12.x code repository
  (if deprecated it will prompt warning message so you can abort the bash script ASAP)
  ```
  curl -sL https://deb.nodesource.com/setup_12.x | sudo -E bash -l
  ```
* install by running `apt-get install -y nodejs`
* check the version after installation : `nodejs --version`

#### Yarn
Yarn is a Javascript package management tool, it can be installed by :
* add new key for the code repository
  ```
  curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
  ```
* add download URL to PPA repo
  ```
  echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
  ```
* update and install by `apt-get update` and `apt-get install yarn`
