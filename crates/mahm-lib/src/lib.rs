#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, CString};
use std::ptr;

use mahm::installation_path;
use mahm::monitor::entry::Entry;
use mahm::monitor::header::Header;
use mahm::monitor::HardwareMonitor;

fn string_to_raw(s: &str) -> *const c_char {
  CString::new(s).unwrap().into_raw()
}

fn drop_raw(s: *mut c_char) {
  if !s.is_null() {
    drop(unsafe { CString::from_raw(s) })
  }
}

#[no_mangle]
pub extern "C" fn mahm_version() -> *const c_char {
  CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn mahm_installation_path() -> *const c_char {
  match installation_path() {
    Ok(path) => string_to_raw(&path),
    Err(_) => ptr::null(),
  }
}

#[no_mangle]
pub unsafe extern "C" fn mahm_free_string(s: *mut c_char) {
  drop_raw(s);
}

#[repr(C)]
pub struct FfiHardwareMonitor {
  raw: *mut HardwareMonitor<'static>,

  pub header: *const FfiHeader,
  pub entries: *const FfiEntry,
  pub entry_count: u32,
}

impl Drop for FfiHardwareMonitor {
  fn drop(&mut self) {
    unsafe { drop(Box::from_raw(self.raw)) }
  }
}

#[repr(C)]
pub struct FfiHeader {
  pub signature: *const c_char,
  pub version: *const c_char,
  pub gpu_entry_count: u32,
  pub entry_count: u32,
  pub time: i64,
}

impl Drop for FfiHeader {
  fn drop(&mut self) {
    drop_raw(self.signature as *mut _);
    drop_raw(self.version as *mut _);
  }
}

#[repr(C)]
pub struct FfiEntry {
  pub src_name: *const c_char,
  pub src_units: *const c_char,
  pub localized_src_name: *const c_char,
  pub localized_src_units: *const c_char,
  pub recommended_format: *const c_char,
  pub data: f32,
  pub min_limit: f32,
  pub max_limit: f32,
  pub flags: u32,
  pub gpu: u32,
  pub src_id: u32,
}

impl Drop for FfiEntry {
  fn drop(&mut self) {
    drop_raw(self.src_name as *mut _);
    drop_raw(self.src_units as *mut _);
    drop_raw(self.localized_src_name as *mut _);
    drop_raw(self.localized_src_units as *mut _);
    drop_raw(self.recommended_format as *mut _);
  }
}

#[no_mangle]
pub extern "C" fn mahm_create_hardware_monitor() -> *mut FfiHardwareMonitor {
  Box::into_raw(Box::new(FfiHardwareMonitor {
    raw: Box::into_raw(Box::new(HardwareMonitor::new())),
    header: ptr::null(),
    entries: ptr::null(),
    entry_count: 0,
  }))
}

#[no_mangle]
pub unsafe extern "C" fn mahm_destroy_hardware_monitor(hm: *mut FfiHardwareMonitor) {
  drop(Box::from_raw(hm));
}

fn header_into_ffi(header: &Header) -> FfiHeader {
  FfiHeader {
    signature: string_to_raw(&header.signature()),
    version: string_to_raw(&header.version()),
    gpu_entry_count: header.gpu_entry_count,
    entry_count: header.entry_count,
    time: header.time.timestamp(),
  }
}

fn entry_into_ffi(entry: &Entry) -> FfiEntry {
  FfiEntry {
    src_name: string_to_raw(&entry.src_name),
    src_units: string_to_raw(&entry.src_units),
    localized_src_name: string_to_raw(&entry.localized_src_name),
    localized_src_units: string_to_raw(&entry.localized_src_units),
    recommended_format: string_to_raw(&entry.recommended_format),
    data: entry.data,
    min_limit: entry.min_limit,
    max_limit: entry.max_limit,
    flags: entry.flags.bits(),
    gpu: entry.gpu,
    src_id: entry.src_id,
  }
}

#[no_mangle]
pub unsafe extern "C" fn mahm_refresh(hm: *mut FfiHardwareMonitor) -> bool {
  let hm = &mut *hm;
  let raw = &mut *hm.raw;

  match raw.refresh() {
    Ok(()) => {
      let header = raw.header.as_ref().map(|h| header_into_ffi(h));
      let entries = raw.entries.iter().map(|e| entry_into_ffi(e)).collect::<Vec<_>>();

      hm.header = match header {
        Some(h) => Box::into_raw(Box::new(h)),
        None => ptr::null(),
      };

      hm.entry_count = entries.len() as u32;
      hm.entries = match entries.len() {
        0 => ptr::null(),
        _ => Box::into_raw(entries.into_boxed_slice()) as *const _,
      };

      true
    }
    Err(_) => false,
  }
}
