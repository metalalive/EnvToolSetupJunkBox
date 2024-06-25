
## Node.js and NPM
Manual installation with pre-built binary
- Download from this [official distro page](https://nodejs.org/dist/)
  - For older OS (e.g. Debian 9), the node.js is limited to version `17.x.x` (or older version)
- extract downloaded file, add environment variable `NODE_HOME` to bash file at `/home/your-user-name/.bashrc`
  ```bash
  NODE_HOME="/path/to/node-porebuilt-linux-x64"; export NODE_PATH;
  PATH="${NODE_HOME}/bin:${PATH}"; export PATH;
  ```
- reopen the terminal, try the commands which should return version string
  - `node --version`
  - `npm --version`

## Frequently used npm commands
List available versions excluding alpha / beta
```bash
npm show <your-package-name>@* version
```
List all available versions including alpha / beta
```bash
npm view <your-package-name> versions
```
Install specific version of npm package
```bash
npm install <your-package-name>@<specific-version>
```
Uninstall npm package
```bash
npm uninstall <your-package-name> 
```
List installed npm packages globally or locally
```bash
npm ls --location=local
npm ls --location=global
```

## Swagger
- clone [`swagger-editor`](https://github.com/swagger-api/swagger-editor) and [`swagger-ui`](https://github.com/swagger-api/swagger-ui) from the respective repositories
- switch to `swagger-editor` folder, install `http-server` module
- for swagger editor
  - run the command `http-server` to start an internal server.
  - visit `localhost:8080` in your web browser, you should see web user interface of the editor.
  - modify your [Open API specification](https://github.com/OAI/OpenAPI-Specification/tree/main/versions) (should be saved in `.json` or `.yaml`) using the editor
- once your Open API specification is ready, install it using `swagger-ui`
  - prepare another HTTP server for the API documentation page, copy entire `swagger-ui` folder to appropriate path for the server
  - switch to the `swagger-ui` folder, open `dist/swagger-initializer.js`
  - replace `url` attribute with the path to your Open API spec, in the instantiation of class `SwaggerUIBundle`
  - visit the path to the index page of the `swagger-ui` in your web browser, your Open API spec should be peoperly renderred in HTML / CSS

## Reference
- [Setup -- Swagger Editor Documentation](https://swagger.io/docs/open-source-tools/swagger-editor/)
- [Create a Swagger UI display with an OpenAPI spec document](https://idratherbewriting.com/learnapidoc/pubapis_swagger.html#create_swaggerui)
