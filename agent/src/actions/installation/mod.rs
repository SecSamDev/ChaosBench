#[cfg(target_os="windows")]
pub mod win;
use chaos_core::{action::PackageActionType, err::ChaosResult, parameters::TestParameters};
#[cfg(target_os="windows")]
pub use win::*;

#[cfg(target_os="linux")]
pub mod linux;
#[cfg(target_os="linux")]
pub use linux::*;


pub fn package_action(action : &PackageActionType, parameters: &TestParameters) -> ChaosResult<()> {
    match action {
        PackageActionType::Install => execute_install(parameters),
        PackageActionType::Uninstall => execute_uninstall(parameters),
        PackageActionType::InstallWithError => execute_install_with_error(parameters),
        PackageActionType::IsInstalled => check_installed(parameters),
        PackageActionType::IsNotInstalled => check_not_installed(parameters),
    }
}