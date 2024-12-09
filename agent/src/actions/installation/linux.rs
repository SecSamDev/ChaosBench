use chaos_core::{action::install::{InstallCheckParameters, InstallParameters, InstallWithErrorParameters}, err::{ChaosError, ChaosResult}, parameters::TestParameters};

use crate::api::download_file;


pub fn execute_install(parameters: &TestParameters) -> ChaosResult<()> {
    log::info!("Executing install");
    let parameters: InstallParameters = parameters.try_into()?;
    let mut command = std::process::Command::new(r"dpkg");
    let file_location = download_file(&parameters.installer)?;
    command
        .arg("-i")
        .arg(&file_location.to_string_lossy()[..]);
    for (param, value) in &parameters.parameters {
        command.arg(format!("{}={}", param, value));
    }
    let output = match command.output() {
        Ok(v) => v,
        Err(_) => {
            return Err(ChaosError::Other(format!(
                "Cannot install {}", &parameters.installer
            )))
        }
    };
    let exit_code = output.status.code().unwrap_or(-1);
    if output.status.success() {
        if output.status.success() {
            log::info!("Installed {}", parameters.installer);
            return Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            log::error!("Error installing (stdout):\n{}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr[..]);
            log::error!("Error installing (stderr):\n{}", stderr);
        }
    }
    Err(ChaosError::Other(format!(
        "Cannot install {}_ exit_status={}", &parameters.installer, exit_code
    )))
}

pub fn execute_uninstall(parameters: &TestParameters)-> ChaosResult<()> {
    log::info!("Executing uninstall");
    let parameters: InstallParameters = parameters.try_into()?;
    let mut command = std::process::Command::new(r"dpkg");
    let file_location = download_file(&parameters.installer)?;
    command
        .arg("-r")
        .arg(&file_location.to_string_lossy()[..]);
    for (param, value) in &parameters.parameters {
        command.arg(format!("{}={}", param, value));
    }
    let output = match command.output() {
        Ok(v) => v,
        Err(_) => {
            return Err(ChaosError::Other(format!(
                "Cannot uninstall {}", &parameters.installer
            )))
        }
    };
    let exit_code = output.status.code().unwrap_or(-1);
    if output.status.success() {
        if output.status.success() {
            log::info!("Uninstalled {}", parameters.installer);
            return Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            log::error!("Error uninstalling (stdout):\n{}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr[..]);
            log::error!("Error uninstalling (stderr):\n{}", stderr);
        }
    }
    Err(ChaosError::Other(format!(
        "Cannot uninstall {}_ exit_status={}", &parameters.installer, exit_code
    )))
}

pub fn execute_install_with_error(parameters: &TestParameters) -> ChaosResult<()>{
    let parameters: InstallWithErrorParameters = parameters.try_into()?;
    let mut command = std::process::Command::new(r"dpkg");
    let file_location = download_file(&parameters.installer)?;
    command
        .arg("-i")
        .arg(&file_location.to_string_lossy()[..]);
    for (param, value) in &parameters.parameters {
        command.arg(format!("{}={}", param, value));
    }
    let install_status = command.status().unwrap().code().unwrap_or_default();
    assert_eq!(parameters.error, install_status);
    Ok(())
}

fn list_deb_packages() -> String {
    let stdout = match std::process::Command::new("dpkg").arg("-l").output() {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    String::from_utf8_lossy(&stdout.stdout).to_string()
}

fn deb_package_with_name<'a>(name: &'a str, data: &'a str) -> Option<&'a str> {
    for line in data.lines() {
        if line.contains(name) {
            let splited = line.split(' ');
            let mut first = false;
            for name in splited {
                if name.is_empty() {
                    continue;
                }
                if !first {
                    first = true;
                    continue;
                }
                return Some(name);
            }
            return None;
        }
    }
    None
}

fn list_rpm_packages() -> String {
    let stdout = match std::process::Command::new("rpm").arg("-qa").output() {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    String::from_utf8_lossy(&stdout.stdout).to_string()
}

fn rpm_package_with_name<'a>(name: &'a str, data: &'a str) -> Option<&'a str> {
    for line in data.lines() {
        if line.contains(name) {
            let last_pos = match line.rfind('-') {
                Some(v) => v,
                None => continue,
            };
            if last_pos == 0 {
                continue;
            }
            let line = &line[0..last_pos];
            let last_pos = match line.rfind('-') {
                Some(v) => v,
                None => continue,
            };
            if last_pos == 0 {
                continue;
            }
            return Some(&line[0..last_pos]);
        }
    }
    None
}

/// Returns Ok(()) when the product IS installed
pub fn check_installed(parameters: &TestParameters) -> ChaosResult<()>{
    let parameters: InstallCheckParameters = parameters.try_into()?;
    let rpm_packages = list_rpm_packages();
    if let Some(_rpm) = rpm_package_with_name(&parameters.product_name, &rpm_packages) {
        return Ok(())
    }
    let deb_packages = list_deb_packages();
    if let Some(_deb) = deb_package_with_name(&parameters.product_name, &deb_packages) {
        return Ok(())
    }
    Err(ChaosError::Other("Product is not installed".into()))
}

/// Returns Ok(()) when the product IS NOT installed
pub fn check_not_installed(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: InstallCheckParameters = parameters.try_into()?;
    let rpm_packages = list_rpm_packages();
    if let Some(_rpm) = rpm_package_with_name(&parameters.product_name, &rpm_packages) {
        return Ok(())
    }
    let deb_packages = list_deb_packages();
    if let Some(_deb) = deb_package_with_name(&parameters.product_name, &deb_packages) {
        return Ok(())
    }
    Err(ChaosError::Other("Product is not installed".into()))
}

#[test]
fn should_extract_correctly_deb_package_name() {
    let packages = r#"Desired=Unknown/Install/Remove/Purge/Hold
| Status=Not/Inst/Conf-files/Unpacked/halF-conf/Half-inst/trig-aWait/Trig-pend
|/ Err?=(none)/Reinst-required (Status,Err: uppercase=bad)
||/ Name                          Version                                 Architecture Description
+++-=============================-=======================================-============-================================================================================
ii  adduser                       3.118ubuntu5                            all          add and remove users and groups
ii  apparmor                      3.0.4-2ubuntu2.3                        amd64        user-space parser utility for AppArmor
ii  apport                        2.20.11-0ubuntu82.5                     all          automatically generate crash reports for debugging
ii  linux-libc-dev:amd64          5.15.0-117.127                          amd64        Linux Kernel Headers for development
ii  llvm                          1:14.0-55~exp2                          amd64        Low-Level Virtual Machine (LLVM)
ii  llvm-14                       1:14.0.0-1ubuntu1.1                     amd64        Modular compiler and toolchain technologies
ii  llvm-14-dev                   1:14.0.0-1ubuntu1.1                     amd64        Modular compiler and toolchain technologies, libraries and headers
ii  llvm-14-linker-tools          1:14.0.0-1ubuntu1.1                     amd64        Modular compiler and toolchain technologies - Plugins
ii  llvm-14-runtime               1:14.0.0-1ubuntu1.1                     amd64        Modular compiler and toolchain technologies, IR interpreter
ii  llvm-14-tools                 1:14.0.0-1ubuntu1.1                     amd64        Modular compiler and toolchain technologies, tools
ii  llvm-runtime:amd64            1:14.0-55~exp2                          amd64        Low-Level Virtual Machine (LLVM), bytecode interpreter
ii  locales                       2.35-0ubuntu3.8                         all          GNU C Library: National Language (locale) data [support]
ii  login                         1:4.8.1-2ubuntu2.2                      amd64        system login tools
ii  logrotate                     3.19.0-1ubuntu1.1                       amd64        Log rotation utility
ii  logsave                       1.46.5-2ubuntu1.1                       amd64        save the output of a command in a log file
ii  lsb-base                      11.1.0ubuntu4                           all          Linux Standard Base init script functionality
ii  lsb-release                   11.1.0ubuntu4                           all          Linux Standard Base version reporting utility
ii  lshw                          02.19.git.2021.06.19.996aaad9c7-2build1 amd64        information about hardware configuration
ii  lsof                          4.93.2+dfsg-1.1build2                   amd64        utility to list open files
ii  lto-disabled-list             24                                      all          list of packages not to build with LTO
ii  make                          4.3-4.1build1                           amd64        utility for directing compilation
ii  man-db                        2.10.2-1                                amd64        tools for reading manual pages
ii  manpages                      5.10-1ubuntu1                           all          Manual pages about using a GNU/Linux system
ii  manpages-dev                  5.10-1ubuntu1                           all          Manual pages about using GNU/Linux for development
ii  mawk                          1.3.4.20200120-3                        amd64        Pattern scanning and text processing language
ii  media-types                   7.0.0                                   all          List of standard media types and their usual file extension
ii  chaos-bench                   0.1.0                                   amd64        ChaosBench
ii  motd-news-config              12ubuntu4.5                             all          Configuration for motd-news shipped in base-files
ii  mount                         2.37.2-4ubuntu3.4                       amd64        tools for mounting and manipulating filesystems
ii  mtr-tiny                      0.95-1                                  amd64        Full screen ncurses traceroute tool
ii  nano                          6.2-1                                   amd64        small, friendly text editor inspired by Pico
"#;
    let package_name = deb_package_with_name("chaos-bench", packages).unwrap();
    assert_eq!("chaos-bench", package_name);
}

#[test]
fn should_extract_correctly_rpm_package_name() {
    let packages = r#"libgcc-11.4.1-3.el9.x86_64
fonts-filesystem-2.0.5-7.el9.1.noarch
linux-firmware-whence-20240219-143.el9.noarch
crypto-policies-20240202-1.git283706d.el9.noarch
hwdata-0.348-9.13.el9.noarch
mingw64-gcc-13.2.1-7.el9.x86_64
mingw64-gcc-c++-13.2.1-7.el9.x86_64
chaos-bench-0.1.0~rc1-1.el9.x86_64
xkeyboard-config-2.33-2.el9.noarch
tzdata-2024a-1.el9.noarch
liberation-fonts-common-2.1.3-5.el9.noarch
hyperv-daemons-license-0-0.42.20190303git.el9.noarch
gnome-control-center-filesystem-40.0-30.el9.noarch
abattis-cantarell-fonts-0.301-4.el9.noarch
cups-filesystem-2.3.3op2-24.el9.noarch
mozilla-filesystem-1.9-30.el9.x86_64
foomatic-db-filesystem-4.0-72.20210209.el9.noarch
google-noto-cjk-fonts-common-20230817-2.el9.noarch
"#;
    let package_name = rpm_package_with_name("chaos-bench", packages).unwrap();
    assert_eq!("chaos-bench", package_name);
}