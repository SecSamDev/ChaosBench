use std::io::Write;
use chaos_core::{action::{dns::DnsParameters, DnsActionType}, err::{ChaosError, ChaosResult}, parameters::TestParameters};

#[cfg(target_os="linux")]
const ETC_HOST_LOCATION : &str = "/etc/hosts";
#[cfg(target_os="windows")]
const ETC_HOST_LOCATION : &str = r"C:\Windows\System32\drivers\etc\hosts";

#[cfg(target_os="linux")]
const NEW_LINE : &[u8] = b"\n";
#[cfg(target_os="windows")]
const NEW_LINE : &[u8] = b"\r\n";

pub fn dns_action(action : &DnsActionType, parameters: &TestParameters) -> ChaosResult<()> {
    match action {
        DnsActionType::Add => add_dns(parameters),
        DnsActionType::Remove => remove_dns(parameters),
    }
}

pub fn add_dns(parameters : &TestParameters) -> ChaosResult<()> {
    log::info!("Adding DNS entry in /etc/hosts");
    let parameters: DnsParameters = parameters.try_into()?;
    let mark = gen_mark(&parameters);
    let content = format!("{}\t{}\t# {mark}", parameters.ip, parameters.domain);
    let mut etc_file = read_etc_file()?;
    if check_if_mark(&etc_file, &mark) {
        return Ok(())
    }
    etc_file.push(content);
    write_etc_file(&etc_file)
}

pub fn remove_dns(parameters : &TestParameters) -> ChaosResult<()> {
    log::info!("Removing DNS entry in /etc/hosts");
    let parameters: DnsParameters = parameters.try_into()?;
    let mark = gen_mark(&parameters);
    let etc_file = read_etc_file()?;
    let mut new_etc_file = Vec::with_capacity(etc_file.len());
    for line in etc_file {
        if line.contains(&mark) {
            continue
        }
        new_etc_file.push(line);
    }
    write_etc_file(&new_etc_file)
}

fn check_if_mark(content : &Vec<String>, mark : &str) -> bool {
    for line in content {
        if line.contains(mark) {
            return true
        }
    }
    false
}

fn gen_mark(params : &DnsParameters) -> String {
    format!("MARK-{}-{}", params.ip, params.domain)
}

fn read_etc_file() -> ChaosResult<Vec<String>> {
    let content = std::fs::read_to_string(ETC_HOST_LOCATION).map_err(|e| ChaosError::Other(format!("Cannot read /etc/hosts: {e}")))?;
    Ok(content.lines().map(|v| v.into()).collect())
}

fn write_etc_file(content : &Vec<String>) -> ChaosResult<()> {
    write_etc_file_int(content).map_err(|e| ChaosError::Other(format!("Cannot write to /etc/hosts: {e}")))
}
fn write_etc_file_int(content : &Vec<String>) -> std::io::Result<()> {
    let mut f = std::fs::File::create(ETC_HOST_LOCATION)?;
    for line in content {
        f.write_all(line.as_bytes())?;
        f.write_all(NEW_LINE)?;
    }
    Ok(())
}