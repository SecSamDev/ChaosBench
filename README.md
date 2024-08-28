# ChaosBench

A simple tool for testing application packages across multiple platforms.

```
┌─────────────────────────────────────────────────────────────┐┌────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│Agent logs           Shows an agent logs                     ││______________                     ________                  ______                                                                             │
│Stop agent logs      Stops receiving agent logs              ││__  ____/__  /_______ ________________  __ )____________________  /_                                                                            │
│App logs             Shows app logs of an agent              ││_  /    __  __ \  __ `/  __ \_  ___/_  __  |  _ \_  __ \  ___/_  __ \                                                                           │
│Stop app logs        Stops receiving app logs                ││/ /___  _  / / / /_/ // /_/ /(__  )_  /_/ //  __/  / / / /__ _  / / /                                                                           │
│Create scenario      Creates a new testing scenario          ││\____/  /_/ /_/\__,_/ \____//____/ /_____/ \___//_/ /_/\___/ /_/ /_/                                                                            │
│Start scenario       Starts a testing scenario               │└────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
│Stop scenario        Stops a testing scenario                │┌Agent Logs──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│Edit scenario        Edit parameters of scenario             ││ 2024-03-24 08:02:12 | main | INFO  | Sent completed task                                                                                       │
│List scenarios       List all file scenario                  ││ 2024-03-24 08:02:12 | main | INFO  | Task to execute ID=37 Start=0 Limit=10000 TTL=-1711263722018                                              │
│List test scenarios  List all testing scenarios              ││ 2024-03-24 08:02:11 | main | INFO  | NextTask(AgentTask { id: 37, scene_id: 3, agent: "bccbdf7d-093e-911b-a2c0-047c16c20e40", limit: 10000, pre│
│Backup               Saves the server state                  ││ 2024-03-24 08:02:11 | main | INFO  | Sent completed task                                                                                       │
│Report               Generates a markdown report             ││ 2024-03-24 08:02:11 | main | INFO  | Task to execute ID=36 Start=0 Limit=10000 TTL=-1711263721195                                              │
│Exit                 Exists the interface                    ││ 2024-03-24 08:02:10 | main | INFO  | NextTask(AgentTask { id: 36, scene_id: 3, agent: "bccbdf7d-093e-911b-a2c0-047c16c20e40", limit: 10000, pre│
│                                                             ││ 2024-03-24 08:02:10 | main | INFO  | Sent completed task                                                                                       │
│                                                             ││ 2024-03-24 08:02:10 | main | INFO  | Task to execute ID=35 Start=0 Limit=10000 TTL=-1711263720340                                              │
│                                                             ││ 2024-03-24 08:02:10 | main | INFO  | NextTask(AgentTask { id: 35, scene_id: 3, agent: "bccbdf7d-093e-911b-a2c0-047c16c20e40", limit: 10000, pre│
│                                                             ││ 2024-03-24 08:02:09 | main | INFO  | Sent completed task                                                                                       │
│                                                             ││ 2024-03-24 08:02:09 | main | INFO  | Task to execute ID=34 Start=0 Limit=30000 TTL=-1711263699522                                              │
│                                                             ││ 2024-03-24 08:02:09 | main | INFO  | NextTask(AgentTask { id: 34, scene_id: 3, agent: "bccbdf7d-093e-911b-a2c0-047c16c20e40", limit: 30000, pre│
│                                                             ││ 2024-03-24 08:02:08 | main | INFO  | Sent completed task                                                                                       │
│                                                             ││ 2024-03-24 08:02:08 | main | INFO  | Task to execute ID=33 Start=0 Limit=30000 TTL=-1711263698697                                              │
│                                                             ││ 2024-03-24 08:02:08 | main | INFO  | NextTask(AgentTask { id: 33, scene_id: 3, agent: "bccbdf7d-093e-911b-a2c0-047c16c20e40", limit: 30000, pre│
│                                                             ││ 2024-03-24 08:02:07 | main | INFO  | Sent completed task                                                                                       │
│                                                             ││ 2024-03-24 08:02:07 | main | INFO  | Task to execute ID=32 Start=0 Limit=30000 TTL=-1711263697882                                              │
│                                                             ││ 2024-03-24 08:02:07 | main | INFO  | NextTask(AgentTask { id: 32, scene_id: 3, agent: "bccbdf7d-093e-911b-a2c0-047c16c20e40", limit: 30000, pre│
│                                                             ││ 2024-03-24 08:02:07 | main | INFO  | Sent completed task                                                                                       │
│                                                             ││ 2024-03-24 08:02:07 | main | INFO  | Task to execute ID=31 Start=0 Limit=30000 TTL=-1711263697067                                              │
└─────────────────────────────────────────────────────────────┘│ 2024-03-24 08:02:06 | main | INFO  | NextTask(AgentTask { id: 31, scene_id: 3, agent: "bccbdf7d-093e-911b-a2c0-047c16c20e40", limit: 30000, pre│
┌─────────────────────────────────────────────────────────────┐│ 2024-03-24 08:02:06 | main | INFO  | Sent completed task                                                                                       │
│Start scenario OK                                            ││ 2024-03-24 08:02:06 | main | INFO  | Task to execute ID=30 Start=0 Limit=30000 TTL=-1711263696248                                              │
│Pruebas Full                                                 │└38/38───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
│Scenario ID?                                                 │┌App Logs────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│Stop scenario OK                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│Scenario ID?                                                 ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
│                                                             ││                                                                                                                                                │
└─────────────────────────────────────────────────────────────┘└────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```


# Index
1. [Scenario design](#scenario-design)
2. [Report generation](#report-gen)
3. [Intercept HTTP requests](#http-intercept)
4. [Building](#building)

## Scenario design<a id="scenario-design"></a>

```yaml
name: All tests
description: All tests
parameters:
  app_version: "1.2.0"
  targets: 
    - "arch:x86"
    - "arch:x64"
    - "name!:.*Debug.*"
  user_name: CNCMS\TestUser # User to start interactive session
  user_password: TestUser123$ # Password to start interactive session
  # Name of the application service. Used by StartAppService and StartService
  service_name: superprogram 

  install_parameters:
    SERVER: 10.0.0.2:443
    API_KEY: 12345
  # Test that installation parameters give an error
  install_error_parameters:
    SERVER: not_a_hostname:123:123
    API_KEY: null
  # (Optional) extra command line after /i xxx.msi /qn ...
  install_command: /lv install_log.txt SERVER=10.0.0.2 API_KEY=12345 
  # Custom command for desinstallation
  uninstall_command: ... 
  # Custom parameters for desinstallation
  uninstall_parameters:
    force: true
  # All files and folders used by our application that needs to be cleaned after
  application_folders:
    - C:\Program Files\program
    - C:\ProgramData\program
    - "%APPDATA%\\Temp\\program"
  # https://serverfault.com/questions/813506/setting-environment-variable-for-service
  service_env_vars: # Custom env vars for the application service
    TEMP: C:\ProgramData\chaos\app_temp
    TMP: C:\ProgramData\chaos\app_temp
  user_env_vars: # Custom env vars for the user session
    TEMP: C:\ProgramData\chaos\user_temp
    TMP: C:\ProgramData\chaos\user_temp

variables:
  application_folder: C:\Program Files\program
files:
  - "Superprogram file.exe"
actions:
  - name: UninstallWithUninstaller
    action: Execute
    parameters:
      command: "$application_folder\\uninstaller.exe --force"

scenario:
  cleanup:
    phase:
      - RestartMachine

scene_preparation:
  phase_timeout: 10s
  cleanup:
    actions:
      - Uninstall
      - CleanFolders
      - ResetEnvVars
      - CloseUserSession
  before: # Before executing a scenario
    actions:
      - CleanFolders # Clean all application folders/files
      - SetupEnvVars
  after_first: # After executing the first phase
    actions: []
  before_last: # Before executing the last phase
    actions:
      - ExtractApplicationData
  after: # After executing a scenario
    actions:
      - CleanTmpFolder
      - ResetEnvVars
      - CloseUserSession

scenes:
  - name: Simple Install/Uninstall
    description: The app must be installed and uninstalled
    phases:
      - Install
      - StartUserSession
      - Uninstall

  - name: Install y Uninstall desde uninstaller.exe
    description: The app must be installed and uninstalled using uninstall.exe
    phases:
      - Install
      - UninstallWithUninstaller
  - name: Instalar y actualizar a través del servidor
    description: The app must be updating by the Server (custom command)
    phases:
      - Install
      - UpdateByWeb # Custom command
      - RestartService
  - name: Instalar e instalar encima
    description: The app must be installed and cannot be installed if it's alredy installed
    phases:
      - Install
      - InstallAlredyInstalledError # Install when its alredy installed. Must give error
      - InstallParameterError # Install with erroneous parameters. Must give error
```

## Report Generation<a id="report-gen"></a>

<details>
<summary>Show Report</summary>

># Pruebas Full
>
>## Simple Install/Uninstall
>
><details>
><summary>Show test</summary>
>
>|ID|State|Action|Agent|Hostname|Error|
>|-----|-----|-----|-----|-----|-----|
>|0|❌|CleanFolders|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1|Custom action CleanFolders not found|
>|1|❌|SetupEnvVars|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1|Custom action SetupEnvVars not found|
>|2|❌|Install|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1|Installer name "$installer" not found|
>|3|✅|StartUserSession|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1||
>|4|❌|ExtractApplicationData|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1|Custom action ExtractApplicationData not found|
>|5|❌|Uninstall|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1|Installer name "$installer" not found|
>|6|✅|CleanTmpFolder|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1||
>|7|❌|ResetEnvVars|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1|Custom action ResetEnvVars not found|
>|8|✅|CloseUserSession|db4fd010-52e4-4c30-a52a-d3e4a90b216a|PC-TEST-1||
></details>
>
></details>

## Http interception<a id="http-intercept"></a>

The chaos server allows you to intercept HTTP traffic and apply scripts to validate that the requests and responses from the application being tested are as expected.
This can be achieved with the Actions HttpRequest and HttpResponse which forwards the petition to the real server.
The scripting language is *Rhai* to simplify the compilation process. In the scope of the executed script the request body is seted as "body", the headers as a map of strings named "headers" and the status code of the response as "status_code". Depending on the content type the body will be a serde_json::Value or a Vec\<u8\>.
For the interception to work properly the application must include the SSL certificate of the chaos server.

## Building<a id="building"></a>
The project uses a custom cargo command: [Xtask](https://github.com/matklad/cargo-xtask) to build all the components.

### Preparation
The server address is statically setted in the agent as to not have a configuration file. Create the file `.cargo/config.toml` with the following content:

```toml
[env]
CA_CERT = "..\\..\\.cargo\\myCA.pem"
SERVER_CERTIFICATE = "..\\..\\.cargo\\certs.crt"
SERVER_KEY = "..\\..\\.cargo\\key.key"
SERVER_PORT = "443"
SERVER_ADDRESS = "security.com"

[alias]
xtask = "run --package xtask --"
```

The certificate and keys can be generated with:

```
openssl genrsa -out myCA.key 2048
openssl req -x509 -new -nodes -key myCA.key -sha256 -days 10000 -out myCA.pem -subj "/C=ES/ST=Madrid/L=Madrid/O=SecSamDev/OU=Central/CN=secsamdev.com/emailAddress=samuel.garces@protonmail.com"
openssl genrsa -out key.key 2048
openssl req -new -key key.key -out test_server.csr -subj "/C=ES/ST=Madrid/L=Madrid/O=SecSamDev/OU=Security/CN=security.secsamdev.com/emailAddress=samuel.garces@protonmail.com"
openssl x509 -req -in test_server.csr -CA myCA.pem -CAkey myCA.key -CAcreateserial -out certs.crt -days 10000 -sha256 -extfile test_server.ext
```

### Build Agent
This only builds the agent, not the installer. To use as a standalon executable for testing:

`cargo xtask build-agent --target-dir "~\BuildDir\ChaosBench\Agent" --no-service`

For production (windows service):
`cargo xtask build-agent --target-dir "~\BuildDir\ChaosBench\Agent"`

### Build Agent installer

For the installer its needed [WixV3](https://wixtoolset.org/docs/wix3/) and [cargo-wix](https://github.com/volks73/cargo-wix)

`cargo xtask build-installer --target-dir "~\BuildDir\ChaosBench\Agent"`

### Build server

`cargo xtask build-server --target-dir "~\BuildDir\ChaosBench\Server"`

### Build user cli

`cargo xtask build-user --target-dir "~\BuildDir\ChaosBench\User"`


### Win7 Support

Use `--support-win7` and make sure you have version rust toolchain with version 1.76 installed.
