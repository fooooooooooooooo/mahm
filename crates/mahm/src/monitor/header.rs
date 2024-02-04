use std::fmt;

use chrono::NaiveDateTime;
use mahm_sys::bindings::MAHM_SHARED_MEMORY_HEADER;

#[derive(thiserror::Error, Debug)]
pub enum InvalidHeaderError {
  #[error("memory dead")]
  Dead,
  #[error("invalid signature: 0x{0:X}")]
  InvalidSignature(u32),
}

pub struct Header<'a> {
  raw: &'a MAHM_SHARED_MEMORY_HEADER,
  pub header_size: u32,
  pub gpu_entry_count: u32,
  pub gpu_entry_size: u32,
  pub entry_count: u32,
  pub entry_size: u32,
  pub time: NaiveDateTime,
}

impl<'a> Header<'a> {
  /// # Safety
  /// `raw` must be a valid pointer to a [`MAHM_SHARED_MEMORY_HEADER`].
  pub unsafe fn new(raw: *const MAHM_SHARED_MEMORY_HEADER) -> Result<Self, InvalidHeaderError> {
    const EXPECTED_SIGNATURE: u32 = u32::from_be_bytes([b'M', b'A', b'H', b'M']);
    const DEAD_SIGNATURE: u32 = 0xDEAD;

    let raw = unsafe { &*raw };

    match raw.dwSignature {
      EXPECTED_SIGNATURE => {}
      DEAD_SIGNATURE => return Err(InvalidHeaderError::Dead),
      _ => return Err(InvalidHeaderError::InvalidSignature(raw.dwSignature)),
    }

    let header_size = raw.dwHeaderSize;
    let gpu_entry_count = raw.dwNumGpuEntries;
    let gpu_entry_size = raw.dwGpuEntrySize;
    let entry_count = raw.dwNumEntries;
    let entry_size = raw.dwEntrySize;

    let time = NaiveDateTime::from_timestamp_opt(i64::from(raw.time), 0).unwrap();

    Ok(Self {
      raw,
      header_size,
      gpu_entry_count,
      gpu_entry_size,
      entry_count,
      entry_size,
      time,
    })
  }

  pub fn signature(&self) -> String {
    let signature = self.raw.dwSignature.to_be_bytes();
    String::from_utf8_lossy(&signature).to_string()
  }

  pub fn version(&self) -> String {
    let major = self.raw.dwVersion >> 16;
    let minor = self.raw.dwVersion & 0xFFFF;

    format!("{major}.{minor}")
  }
}

impl<'a> fmt::Debug for Header<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Header")
      .field("signature", &self.signature())
      .field("version", &self.version())
      .field("gpu_entry_count", &self.gpu_entry_count)
      .field("gpu_entry_size", &self.gpu_entry_size)
      .field("entry_count", &self.entry_count)
      .field("entry_size", &self.entry_size)
      .field("time", &self.time)
      .finish()
  }
}

impl fmt::Display for Header<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "signature: {}\nversion: {}\ngpu_entry_count: {}\ngpu_entry_size: {}\nentry_count: {}\nentry_size: {}\ntime: {}",
      self.signature(),
      self.version(),
      self.gpu_entry_count,
      self.gpu_entry_size,
      self.entry_count,
      self.entry_size,
      self.time,
    )
  }
}
