# ChaosBench

A simple tool for testing application packages across multiple platforms.

## Scenario design

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

## Building
The project uses a custom cargo command: [Xtask](https://github.com/matklad/cargo-xtask) to build all the components.

### Preparation
The server address is statically setted in the agent as to not have a configuration file. Create the file `.cargo/config.toml` with the following content:

```toml
[env]
AGENT_SERVER_ADDRESS = "127.0.0.1:8080"

[alias]
xtask = "run --package xtask --"
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