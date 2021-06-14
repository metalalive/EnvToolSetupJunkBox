### PIP
#### Basic
```
<PYTHON_VERSION> -m pip --version
<PYTHON_VERSION> -m pip list
<PYTHON_VERSION> -m pip freeze
```
Note the output from [pip freeze](https://pip.pypa.io/en/stable/cli/pip_freeze/) can be stored and used in other virtualenv to install all the required packages

#### Upgrade package
```
<PYTHON_VERSION> -m pip install --upgrade pip
<PYTHON_VERSION> -m pip install --upgrade <PACKAGE_NAME>
<PYTHON_VERSION> -m pip install -U <PACKAGE_NAME>
```

Only check all available versions , without installation. It will report error, which contains all available versions to install.
```
<PYTHON_VERSION>  -m pip install <PACKAGE_NAME>==
```

### Virtual environment
Create new virtual environment :
```
<PYTHON_VERSION>  -m venv <ENV_PATH>
```

switch to the virtual environment  you just created :
```
source <ENV_PATH>/bin/activate
```

leave from the virtual environment
```
deactivate
```
