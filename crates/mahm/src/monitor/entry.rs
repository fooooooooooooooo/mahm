use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;

use bitflags::bitflags;
use mahm_sys::bindings::{
  MAHM_SHARED_MEMORY_ENTRY, MAHM_SHARED_MEMORY_ENTRY_FLAG_SHOW_IN_LCD, MAHM_SHARED_MEMORY_ENTRY_FLAG_SHOW_IN_OSD,
  MAHM_SHARED_MEMORY_ENTRY_FLAG_SHOW_IN_TRAY,
};

bitflags! {
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub struct EntryFlags: u32 {
    const ShowInOsd = MAHM_SHARED_MEMORY_ENTRY_FLAG_SHOW_IN_OSD;
    const ShowInLcd = MAHM_SHARED_MEMORY_ENTRY_FLAG_SHOW_IN_LCD;
    const ShowInTray = MAHM_SHARED_MEMORY_ENTRY_FLAG_SHOW_IN_TRAY;
  }
}

pub struct Entry<'a> {
  _raw: &'a MAHM_SHARED_MEMORY_ENTRY,
  pub src_name: Cow<'a, str>,
  pub src_units: Cow<'a, str>,

  pub localized_src_name: Cow<'a, str>,
  pub localized_src_units: Cow<'a, str>,

  pub recommended_format: Cow<'a, str>,

  pub data: f32,

  pub min_limit: f32,
  pub max_limit: f32,

  pub flags: EntryFlags,
  pub gpu: u32,
  pub src_id: u32,
}

fn ptr_to_string<'a>(ptr: *const i8) -> Cow<'a, str> {
  unsafe { CStr::from_ptr(ptr) }.to_string_lossy()
}

impl<'a> Entry<'a> {
  /// # Safety
  /// `raw` must be a valid pointer to a [`MAHM_SHARED_MEMORY_ENTRY`].
  pub unsafe fn new(raw: *const MAHM_SHARED_MEMORY_ENTRY) -> Self {
    let raw = unsafe { &*raw };

    let src_name = ptr_to_string(raw.szSrcName.as_ptr());
    let src_units = ptr_to_string(raw.szSrcUnits.as_ptr());
    let localized_src_name = ptr_to_string(raw.szLocalizedSrcName.as_ptr());
    let localized_src_units = ptr_to_string(raw.szLocalizedSrcUnits.as_ptr());
    let recommended_format = ptr_to_string(raw.szRecommendedFormat.as_ptr());

    Self {
      _raw: raw,
      src_name,
      src_units,
      localized_src_name,
      localized_src_units,
      recommended_format,
      data: raw.data,
      min_limit: raw.minLimit,
      max_limit: raw.maxLimit,
      flags: EntryFlags::from_bits_truncate(raw.dwFlags),
      gpu: raw.dwGpu,
      src_id: raw.dwSrcId,
    }
  }
}

impl fmt::Debug for Entry<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Entry")
      .field("src_name", &self.src_name)
      .field("src_units", &self.src_units)
      .field("localized_src_name", &self.localized_src_name)
      .field("localized_src_units", &self.localized_src_units)
      .field("recommended_format", &self.recommended_format)
      .field("data", &self.data)
      .field("min_limit", &self.min_limit)
      .field("max_limit", &self.max_limit)
      .field("flags", &self.flags)
      .field("gpu", &self.gpu)
      .field("src_id", &self.src_id)
      .finish()
  }
}

impl fmt::Display for Entry<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}: {:.2} {}", self.localized_src_name, self.data, self.localized_src_units)
  }
}
