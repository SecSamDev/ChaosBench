use std::borrow::BorrowMut;
use chaos_core::err::{ChaosError, ChaosResult};
use windows::{core::{PCWSTR, PWSTR}, Win32::{Foundation::{ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS}, System::Registry::{RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegEnumKeyExW, RegEnumValueW, RegOpenKeyW, RegQueryValueExW, RegSetValueExW, HKEY, KEY_ALL_ACCESS, KEY_WOW64_64KEY, REG_BINARY, REG_DWORD, REG_EXPAND_SZ, REG_MULTI_SZ, REG_OPTION_NON_VOLATILE, REG_QWORD, REG_SZ, REG_VALUE_TYPE}}};

pub fn to_pwstr(val: &str) -> Vec<u16> {
    let mut val = val.encode_utf16().collect::<Vec<u16>>();
    val.push(0);
    val
}

pub struct RegistryEditor {}

pub enum RegValue {
    DWord(u32),
    QWord(u64),
    SZ(String),
    MultiSZ(Vec<String>),
    ExpandSZ(String),
    Binary(Vec<u8>),
}
impl TryFrom<RegValue> for String {
    type Error = &'static str;

    fn try_from(value: RegValue) -> Result<Self, Self::Error> {
        Ok(match value {
            RegValue::DWord(v) => v.to_string(),
            RegValue::QWord(v) => v.to_string(),
            RegValue::SZ(v) => v,
            RegValue::ExpandSZ(v) => v,
            _ => return Err("Cannot cast to String")
        })
    }
}
impl TryFrom<RegValue> for u32 {
    type Error = &'static str;

    fn try_from(value: RegValue) -> Result<Self, Self::Error> {
        Ok(match value {
            RegValue::DWord(v) => v,
            _ => return Err("Cannot cast to u32")
        })
    }
}
impl TryFrom<RegValue> for u64 {
    type Error = &'static str;

    fn try_from(value: RegValue) -> Result<Self, Self::Error> {
        Ok(match value {
            RegValue::DWord(v) => v as u64,
            RegValue::QWord(v) => v,
            _ => return Err("Cannot cast to u64")
        })
    }
}

impl RegistryEditor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn open_key(&self, hkey : HKEY, name : &str) -> ChaosResult<HKEY> {
        unsafe {
            let mut new_key = HKEY(0);
            let new_key_str = to_pwstr(name);
            if let Err(e) = RegOpenKeyW(hkey, PCWSTR(new_key_str.as_ptr()), &mut new_key) {
                return Err(ChaosError::Other(format!("Cannot open key: {}", e)));
            }
            Ok(new_key)
        }
    }

    pub fn read_value(&self, hkey : HKEY, name : &str) -> ChaosResult<RegValue> {
        unsafe {
            let value_name = to_pwstr(name);
            let mut capacity : u32 = 10_000;
            loop {
                let mut readed_data = vec_with_capacity(capacity as usize);
                let mut data_type : REG_VALUE_TYPE = REG_VALUE_TYPE::default();
                if let Err(err) = RegQueryValueExW(hkey, PCWSTR(value_name.as_ptr()),None, Some(&mut data_type),Some(readed_data.as_mut_ptr()), Some(&mut capacity)) {
                    if err.code() == ERROR_MORE_DATA.to_hresult() {
                        continue;
                    }
                    return Err(ChaosError::Other(format!("read_value({}) Win32 error: {}",name, err.code().0)));
                };
                readed_data.resize(capacity as usize, 0);
                return Ok(match data_type {
                    //https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types
                    REG_DWORD => {
                        if capacity != 4 {
                            return Err(bad_format());
                        }
                        let data : [u8; 4] = match readed_data[0..4].try_into() {
                            Ok(v) => v,
                            Err(_) => return Err(bad_format())
                        };
                        RegValue::DWord(u32::from_ne_bytes(data))
                    },
                    REG_QWORD => {
                        if capacity != 8 {
                            return Err(bad_format());
                        }
                        let data : [u8; 8] = match readed_data[0..8].try_into() {
                            Ok(v) => v,
                            Err(_) => return Err(bad_format())
                        };
                        RegValue::QWord(u64::from_ne_bytes(data))
                    },
                    REG_SZ => {
                        let mut u16_vec : Vec<u16> = readed_data[0..capacity as usize].chunks(2).map(|v| (v[1] as u16) << 8 | v[0] as u16).collect();
                        let _ = u16_vec.pop();//Ends with 00
                        RegValue::SZ(String::from_utf16_lossy(&u16_vec))
                    },
                    REG_MULTI_SZ => {
                        let mut returned_strs = Vec::with_capacity(16);
                        let mut txt = Vec::with_capacity(capacity as usize);
                        let mut txt_lngt = 0;
                        for chr in readed_data[0..capacity as usize].chunks(2).map(|v| (v[1] as u16) << 8 | v[0] as u16) {
                            if chr == 0 {
                                if txt_lngt > 0 {
                                    returned_strs.push(String::from_utf16_lossy(&txt[0..txt_lngt]));
                                }else{
                                    returned_strs.push(String::new());
                                }
                                txt_lngt = 0;
                            }else{
                                txt[txt_lngt] = chr;
                                txt_lngt += 1;
                            }
                        }
                        RegValue::MultiSZ(returned_strs)
                    },
                    REG_BINARY => {
                        RegValue::Binary(readed_data)
                    },
                    REG_EXPAND_SZ => {
                        let mut u16_vec : Vec<u16> = readed_data[0..capacity as usize].chunks(2).map(|v| (v[1] as u16) << 8 | v[0] as u16).collect();
                        let _ = u16_vec.pop();//Ends with 00
                        RegValue::ExpandSZ(String::from_utf16_lossy(&u16_vec))
                    },
                    _ => return Err(bad_format())
                });
            }
        }
    }

    pub fn remove_value(&self, hkey : HKEY, value : &str) -> ChaosResult<()> {
        let value = to_pwstr(value);
        match unsafe { RegDeleteValueW(hkey, windows::core::PCWSTR(value.as_ptr())) } {
            Ok(_) => Ok(()),
            Err(e) => Err(ChaosError::Other(format!("Cannot delete value: {}", e)))
        }
    }

    pub fn create_key(&self, hkey : HKEY, path : &str) -> ChaosResult<HKEY> {
        let mut key_handle = HKEY::default();
        let subkey = to_pwstr(path);
        unsafe {
            match RegCreateKeyExW(
                hkey,
                windows::core::PCWSTR(subkey.as_ptr()),
                0,
                windows::core::PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS | KEY_WOW64_64KEY,
                None,
                key_handle.borrow_mut(),
                None,
            ) {
                Ok(_) => Ok(key_handle),
                Err(e) => Err(ChaosError::Other(format!("Cannot create key: {}", e)))
            }
        }
    }
    pub fn enumerate_keys(&self, hkey : HKEY) -> ChaosResult<Vec<String>> {
        unsafe {
            let mut to_ret = Vec::with_capacity(512);
            let mut count = 0;
            loop {
                let mut key_name_capacity : u32 = 1024;
                let mut key_name_buff = [0; 1024];
                if let Err(err) = RegEnumKeyExW(hkey, count, PWSTR(key_name_buff.as_mut_ptr()),&mut key_name_capacity, None, PWSTR::null(), None, None){
                    if err.code() == ERROR_NO_MORE_ITEMS.to_hresult() {
                        break;
                    }
                    return Err(ChaosError::Other(format!("enumerate_values() Win32 error: {}", err.code().0)));
                };
                to_ret.push(from_pwstr(&key_name_buff[0..key_name_capacity as usize]));
                count += 1;
            }
            Ok(to_ret)
        }
    }

    pub fn enumerate_values(&self, hkey : HKEY) -> ChaosResult<Vec<String>> {
        unsafe {
            let mut to_ret = Vec::with_capacity(512);
            let mut count = 0;
            loop {
                let mut key_name_capacity : u32 = 1024;
                let mut key_name_buff = [0; 1024];
            
                let mut value_type : u32 = 0;
                if let Err(err) = RegEnumValueW(hkey, count, PWSTR(key_name_buff.as_mut_ptr()),&mut key_name_capacity, None, Some(&mut value_type) ,None, None){
                    if err.code() == ERROR_NO_MORE_ITEMS.to_hresult() {
                        break;
                    }
                    return Err(ChaosError::Other(format!("enumerate_values() Win32 error: {}", err.code().0)));
                };
                to_ret.push(from_pwstr(&key_name_buff[0..key_name_capacity as usize]));
                count += 1;
            }
            Ok(to_ret)
        }
    }

    pub fn write_value(&self, hkey : HKEY, value : &str, data : RegValue) -> ChaosResult<()> {
        let mut value_name = to_pwstr(value);
        let (content, data_type) = match data {
            RegValue::DWord(v) => (v.to_ne_bytes().to_vec(), REG_DWORD),
            RegValue::QWord(v) => (v.to_ne_bytes().to_vec(), REG_QWORD),
            RegValue::SZ(v) => (u16_to_u8_vec(v.encode_utf16().collect()), REG_SZ),
            RegValue::MultiSZ(v) => (v.into_iter().flat_map(|v| {
                u16_to_u8_vec(v.encode_utf16().collect())
            }).collect(),REG_MULTI_SZ),
            RegValue::ExpandSZ(v) => (u16_to_u8_vec(v.encode_utf16().collect()), REG_SZ),
            RegValue::Binary(v) => (v, REG_SZ),
        };
        unsafe {
            match RegSetValueExW(hkey, PCWSTR(value_name.as_mut_ptr()), 0, data_type, Some(content.as_slice())) {
                Ok(_) => Ok(()),
                Err(e) => Err(ChaosError::Other(format!("Cannot write value: {}", e)))
            }
        }
    }

    pub fn close_key(&self, key : HKEY) {
        match unsafe { RegCloseKey(key) } {
            Ok(_) => {},
            Err(_) => {},
        }
    }
}

pub fn vec_with_capacity(capacity : usize) -> Vec<u8> {
    vec![0; capacity as usize]
}

pub fn bad_format() -> ChaosError {
    ChaosError::Other("Bad Format".into())
}

pub fn from_pwstr(val: &[u16]) -> String {
    String::from_utf16_lossy(val)
}

pub fn u16_to_u8_vec(mut vec: Vec<u16>) -> Vec<u8> {
    unsafe {
        let capacity = vec.capacity();
        let len = vec.len();
        let ptr = vec.as_mut_ptr();
        std::mem::forget(vec);
        Vec::from_raw_parts(ptr as *mut u8, 2 * len, 2 * capacity)
    }
}