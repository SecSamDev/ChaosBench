use std::{io::Write, os::unix::fs::PermissionsExt, path::PathBuf};

use crate::params::{Architecture, BuildParameters};

const CONTROL_FILE : &str = include_str!("CONTROL.txt");
const PRERM_SCRIPT : &str = include_str!("prerm.sh");
const PREINST_SCRIPT : &str = include_str!("preinst.sh");
const POSTINST_SCRIPT : &str = include_str!("postinst.sh");
const POSTRM_SCRIPT : &str = include_str!("postrm.sh");
const AGENT_SERVICE : &str = include_str!("chaosbench.service");

pub fn build_deb_installer(params: BuildParameters) {
    let version = match &params.version {
        Some(v) => v.clone(),
        None => std::env::var("CARGO_PKG_VERSION").expect("Version variable not detected"),
    };
    let package_folder_name = format!("chaosagent_{}_{}", version, deb_architecture(&params));
    let deb_folder = std::path::Path::new(&params.target_dir).join(&package_folder_name);
    if deb_folder.exists() {
        let _ = std::fs::remove_dir_all(&deb_folder);
    }
    let _ = std::fs::create_dir_all(&deb_folder);

    let lib_path = deb_folder.join("var").join("lib").join("chaosbench");
    let bin_path = deb_folder.join("usr").join("bin");
    let log_path = deb_folder.join("var").join("log").join("chaosbench");
    let etc_path = deb_folder.join("etc").join("chaosbench");
    let _ = std::fs::create_dir_all(&lib_path);
    let _ = std::fs::create_dir_all(&log_path);
    let _ = std::fs::create_dir_all(&etc_path);
    let _ = std::fs::create_dir_all(&bin_path);

    let release_dir = std::path::Path::new(&params.target_dir)
        .join(super::super::cargo_target(&params))
        .join("release");
    std::fs::copy(release_dir.join("agent"), bin_path.join("chaosagent"))
        .expect(&format!("Error copying agent"));

    create_deb_files(&params, &version, &deb_folder);
    let mut command = std::process::Command::new("dpkg-deb");
    command.arg("--build").arg("--root-owner-group").arg(&package_folder_name).current_dir(&params.target_dir);
    command.status().expect("DEB file must be built");
}

fn deb_architecture(params: &BuildParameters) -> &str {
    match params.architecture {
        Architecture::X64 => "amd64",
        Architecture::X86 => "i686",
        Architecture::ARM64 => "arm64",
    }
}

fn create_deb_files(params : &BuildParameters, version : &str, deb_folder : &PathBuf) {
    create_control_file(params, version, deb_folder);
    create_service_file(params, deb_folder);
    create_installation_scripts(params, deb_folder);
}

pub fn create_control_file(params : &BuildParameters, version : &str, deb_folder : &PathBuf) {
    let control_content = deb_control_file(params, &version);
    let debian_path = deb_folder.join("DEBIAN");
    let _ = std::fs::create_dir_all(&debian_path);
    let mut control_file = std::fs::File::create(debian_path.join("control")).expect("Cannot create control file");
    control_file.write_all(control_content.as_bytes()).unwrap();
}

pub fn create_service_file(params : &BuildParameters, deb_folder : &PathBuf) {
    let service_content = create_service(params);
    let services_path = deb_folder.join("etc").join("systemd").join("system");
    let _ = std::fs::create_dir_all(&services_path);
    let mut service_file = std::fs::File::create(services_path.join("chaosbench.service")).expect("Cannot create service file");
    service_file.write_all(service_content.as_bytes()).unwrap();
}

pub fn create_installation_scripts(params : &BuildParameters, deb_folder : &PathBuf) {
    let scripts_path = deb_folder.join("DEBIAN");
    let _ = std::fs::create_dir_all(&scripts_path);
    let mut service_file = std::fs::File::create(scripts_path.join("preinst")).expect("Cannot create preints file");
    service_file.write_all(create_preinst(params).as_bytes()).unwrap();
    let mut service_file = std::fs::File::create(scripts_path.join("postinst")).expect("Cannot create postinst file");
    service_file.write_all(create_postinst(params).as_bytes()).unwrap();
    let mut service_file = std::fs::File::create(scripts_path.join("prerm")).expect("Cannot create prerm file");
    service_file.write_all(create_prerm(params).as_bytes()).unwrap();
    let mut service_file = std::fs::File::create(scripts_path.join("postrm")).expect("Cannot create prerm file");
    service_file.write_all(create_postrm(params).as_bytes()).unwrap();

    let _ = std::fs::set_permissions(scripts_path.join("prerm"), PermissionsExt::from_mode(0o0755));
    let _ = std::fs::set_permissions(scripts_path.join("preinst"), PermissionsExt::from_mode(0o0755));
    let _ = std::fs::set_permissions(scripts_path.join("postinst"), PermissionsExt::from_mode(0o0755));
    let _ = std::fs::set_permissions(scripts_path.join("postrm"), PermissionsExt::from_mode(0o0755));
}

pub fn deb_control_file(params: &BuildParameters, version : &str) -> String {
    CONTROL_FILE.replace("%VERSION%", version).replace("%ARCHITECTURE%", deb_architecture(params))
}

pub fn create_preinst(_params: &BuildParameters) -> String {
    PREINST_SCRIPT.to_string()
}

pub fn create_postinst(_params: &BuildParameters) -> String {
    POSTINST_SCRIPT.to_string()
}
pub fn create_postrm(_params: &BuildParameters) -> String {
    POSTRM_SCRIPT.to_string()
}

pub fn create_prerm(_params: &BuildParameters) -> String {
    PRERM_SCRIPT.to_string()
}
pub fn create_service(_params: &BuildParameters) -> String {
    AGENT_SERVICE.to_string()
}