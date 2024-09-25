use chaos_core::{action::install::{InstallCheckParameters, InstallParameters, InstallWithErrorParameters}, err::{ChaosError, ChaosResult}, parameters::TestParameters};
use windows::Win32::System::Registry::HKEY_LOCAL_MACHINE;

use crate::{api::download_file, common::create_file_path_in_workspace, reg::{self, RegValue}};

/// Installs a MSI
pub fn execute_install(parameters: &TestParameters) -> ChaosResult<()> {
    log::info!("Executing install");
    let parameters: InstallParameters = parameters.try_into()?;
    let mut command = std::process::Command::new(r"C:\Windows\System32\msiexec.exe");
    let log_location = create_file_path_in_workspace("install.log");
    let file_location = download_file(&parameters.installer)?;
    command
        .arg("/i")
        .arg(&file_location.to_string_lossy()[..])
        .arg("/qn")
        .arg("/l*v")
        .arg(log_location.as_os_str());
    for (param, value) in &parameters.parameters {
        command.arg(format!("{}={}", param, value));
    }
    let _status = parse_msi_result(command.status().unwrap().code().unwrap_or_default());
    log::info!("Installed {}", parameters.installer);
    Ok(())
}
/// Uninstalls a MSI
pub fn execute_uninstall(parameters: &TestParameters)-> ChaosResult<()> {
    let parameters: InstallParameters = parameters.try_into()?;
    let mut command = std::process::Command::new(r"C:\Windows\System32\msiexec.exe");
    let log_location = create_file_path_in_workspace("uninstall.log");
    let file_location = download_file(&parameters.installer)?;
    command
        .arg("/x")
        .arg(&file_location.to_string_lossy()[..])
        .arg("/qn")
        .arg("/l*v")
        .arg(log_location.as_os_str());
    let _status = parse_msi_result(command.status().unwrap().code().unwrap_or_default());
    log::info!("Uninstalled {}", parameters.installer);
    Ok(())
}

pub fn execute_install_with_error(parameters: &TestParameters) -> ChaosResult<()>{
    let parameters: InstallWithErrorParameters = parameters.try_into()?;
    let mut command = std::process::Command::new(r"C:\Windows\System32\msiexec.exe");
    let log_location = create_file_path_in_workspace("uninstall.log");
    let file_location = download_file(&parameters.installer)?;
    command
        .arg("/x")
        .arg(&file_location.to_string_lossy()[..])
        .arg("/qn")
        .arg("/l*v")
        .arg(log_location.as_os_str());
    for (param, value) in parameters.parameters {
        command.arg(format!("{}={}", param, value));
    }
    let install_status = command.status().unwrap().code().unwrap_or_default();
    assert_eq!(parameters.error, install_status);
    let _status = parse_msi_result(install_status);
    Ok(())
}

/// Returns Ok(()) when the product IS installed
pub fn check_installed(parameters: &TestParameters) -> ChaosResult<()>{
    let parameters: InstallCheckParameters = parameters.try_into()?;
    let registry = reg::RegistryEditor::new();
    if let Some(product_code) = &parameters.product_code {
        if let Ok(v) = check_if_product_code_in_uninstall_registry(&product_code, &registry) {
            if v {
                return Ok(())
            }
        }
    }
    let installed = check_if_product_name_in_uninstall_registry(&parameters.product_name, &registry)?;
    if installed {
        return Ok(())
    }
    Err(ChaosError::Other(format!("Product is not installed")))
}

/// Returns Ok(()) when the product IS NOT installed
pub fn check_not_installed(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: InstallCheckParameters = parameters.try_into()?;
    let registry = reg::RegistryEditor::new();
    if let Some(product_code) = &parameters.product_code {
        if let Ok(v) = check_if_product_code_in_uninstall_registry(&product_code, &registry) {
            if !v {
                return Ok(())
            }
        }
    }
    let installed = check_if_product_name_in_uninstall_registry(&parameters.product_name, &registry)?;
    if !installed {
        return Ok(())
    }
    Err(ChaosError::Other(format!("Product is installed")))
}

fn check_if_product_name_in_uninstall_registry(product_name : &str, registry : &reg::RegistryEditor) -> ChaosResult<bool>{
    if let Ok(v) = check_if_name_in_uninstall_registry(product_name, registry, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall") {
        return Ok(v)
    }
    check_if_name_in_uninstall_registry(product_name, registry, r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall")
}

fn check_if_product_code_in_uninstall_registry(product_code : &str, registry : &reg::RegistryEditor) -> ChaosResult<bool>{
    if let Ok(v) = check_if_code_in_uninstall_registry(product_code, registry, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall") {
        return Ok(v)
    }
    check_if_code_in_uninstall_registry(product_code, registry, r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall")
}

fn check_if_code_in_uninstall_registry(product_code : &str, registry : &reg::RegistryEditor, uninstall_key : &str) -> ChaosResult<bool>{
    let product_code = product_code.trim();
    let uninstall_key = registry.open_key(HKEY_LOCAL_MACHINE, uninstall_key)?;
    let subkeys = registry.enumerate_keys(uninstall_key)?;
    for key in subkeys {
        if key.trim() == product_code {
            registry.close_key(uninstall_key);
            return Ok(true)
        }
    }
    registry.close_key(uninstall_key);
    Ok(false)
}


fn check_if_name_in_uninstall_registry(product_name : &str, registry : &reg::RegistryEditor, uninstall_key : &str) -> ChaosResult<bool>{
    let product_name = product_name.trim();
    let uninstall_key = registry.open_key(HKEY_LOCAL_MACHINE, uninstall_key)?;
    let subkeys = registry.enumerate_keys(uninstall_key)?;
    for key in subkeys {
        let hkey = match registry.open_key(uninstall_key, &key) {
            Ok(v) => v,
            Err(_) => continue
        };
        let value = match registry.read_value(hkey, "DisplayName") {
            Ok(v) => v,
            Err(_) => {
                registry.close_key(hkey);
                continue
            }
        };
        if let RegValue::SZ(display_name) = value {
            if display_name.trim() == product_name {
                registry.close_key(hkey);
                return Ok(true)
            }
        }
        registry.close_key(hkey);
    }
    registry.close_key(uninstall_key);
    Ok(false)
}

pub fn parse_msi_result(status: i32) -> &'static str {
    match status {
        0 => "The action completed successfully.",
        1 => "Incorrect function",
        2 => "The system cannot find the file specified",
        3 => "The system cannot find the path specified",
        4 => "The system cannot open the file",
        5 => "Access denied",
        6 => "The handle is invalid",
        7 => "The storage control blocks were destoryed",
        8 => "Not enough storage is available to process this command.",
        9 => "The storage control block address is invalid.",
        10 => "The environment is incorrect.",
        11 => "An attempt was made to load a program with an incorrect format.",
        12 => "The access code is invalid.",
        13 => "The data is invalid.",
        14 => "Not enough storage is available to complete this operation.",
        // TODO: add more error codes
        87 => "One of the parameters was invalid.",
        120 => "This value is returned when a custom action attempts to call a function that can't be called from custom actions. The function returns the value ERROR_CALL_NOT_IMPLEMENTED.",
        1259 => "If Windows Installer determines a product might be incompatible with the current operating system, it displays a dialog box informing the user and asking whether to try to install anyway. This error code is returned if the user chooses not to try the installation.",
        1601 => "The Windows Installer service couldn't be accessed. Contact your support personnel to verify that the Windows Installer service is properly registered.",
        1602 => "The user canceled installation.",
        1603 => "A fatal error occurred during installation.",
        1604 => "Installation suspended, incomplete.",
        1605 => "This action is only valid for products that are currently installed.",
        1606 => "The feature identifier isn't registered.",
        1607 => "The component identifier isn't registered.",
        1608 => "This is an unknown property.",
        1609 => "The handle is in an invalid state.",
        1610 => "The configuration data for this product is corrupt. Contact your support personnel.",
        1611 => "The component qualifier not present.",
        1612 => "The installation source for this product isn't available. Verify that the source exists and that you can access it.",
        1613 => "This installation package can't be installed by the Windows Installer service. You must install a Windows service pack that contains a newer version of the Windows Installer service.",
        1614 => "The product is uninstalled.",
        1615 => "The SQL query syntax is invalid or unsupported.",
        1616 => "The record field does not exist.",
        1618 => "Another installation is already in progress. Complete that installation before proceeding with this install. For information about the mutex, see _MSIExecute Mutex.",
        1619 => "This installation package couldn't be opened. Verify that the package exists and is accessible, or contact the application vendor to verify that this is a valid Windows Installer package.",
        1620 => "This installation package couldn't be opened. Contact the application vendor to verify that this is a valid Windows Installer package.",
        1621 => "There was an error starting the Windows Installer service user interface. Contact your support personnel.",
        1622 => "There was an error opening installation log file. Verify that the specified log file location exists and is writable.",
        1623 => "This language of this installation package isn't supported by your system.",
        1624 => "There was an error applying transforms. Verify that the specified transform paths are valid.",
        1625 => "This installation is forbidden by system policy. Contact your system administrator.",
        1626 => "The function couldn't be executed.",
        1627 => "The function failed during execution.",
        1628 => "An invalid or unknown table was specified.",
        1629 => "The data supplied is the wrong type.",
        1630 => "Data of this type isn't supported.",
        1631 => "The Windows Installer service failed to start. Contact your support personnel.",
        1632 => "The Temp folder is either full or inaccessible. Verify that the Temp folder exists and that you can write to it.",
        1633 => "This installation package isn't supported on this platform. Contact your application vendor.",
        1634 => "Component isn't used on this machine.",
        1635 => "This patch package couldn't be opened. Verify that the patch package exists and is accessible, or contact the application vendor to verify that this is a valid Windows Installer patch package.",
        1636 => "This patch package couldn't be opened. Contact the application vendor to verify that this is a valid Windows Installer patch package.",
        1637 => "This patch package can't be processed by the Windows Installer service. You must install a Windows service pack that contains a newer version of the Windows Installer service.",
        1638 => "Another version of this product is already installed. Installation of this version can't continue. To configure or remove the existing version of this product, use Add/Remove Programs in Control Panel.",
        1639 => "Invalid command line argument. Consult the Windows Installer SDK for detailed command-line help.",
        1640 => "The current user isn't permitted to perform installations from a client session of a server running the Terminal Server role service.",
        1641 => "The installer has initiated a restart. This message indicates success.",
        1642 => "The installer can't install the upgrade patch because the program being upgraded may be missing or the upgrade patch updates a different version of the program. Verify that the program to be upgraded exists on your computer and that you have the correct upgrade patch.",
        1643 => "The patch package isn't permitted by system policy.",
        1644 => "One or more customizations aren't permitted by system policy.",
        1645 => "Windows Installer doesn't permit installation from a Remote Desktop Connection.",
        1646 => "The patch package isn't a removable patch package.",
        1647 => "The patch isn't applied to this product.",
        1648 => "No valid sequence could be found for the set of patches.",
        1649 => "Patch removal was disallowed by policy.",
        1650 => "The XML patch data is invalid.",
        1651 => "Administrative user failed to apply patch for a per-user managed or a per-machine application that'is in advertised state.",
        1652 => "Windows Installer isn't accessible when the computer is in Safe Mode. Exit Safe Mode and try again or try using system restore to return your computer to a previous state. Available beginning with Windows Installer version 4.0.",
        1653 => "Couldn't perform a multiple-package transaction because rollback has been disabled. Multiple-package installations can't run if rollback is disabled. Available beginning with Windows Installer version 4.5.",
        1654 => "The app that you're trying to run isn't supported on this version of Windows. A Windows Installer package, patch, or transform that has not been signed by Microsoft can't be installed on an ARM computer.",
        3010 => "A restart is required to complete the install. This message indicates success. This does not include installs where the ForceReboot action is run.",
        _ => "Unknown MsiExec status code"
    }
}
