use std::io;

use anyhow::Context;
use mahm::windows::{map_view_of_file, open_file_mapping, FileMapping, MemoryMappedView};
use mahm_sys::bindings::MAHM_SHARED_MEMORY_HEADER;
use windows::Win32::System::Memory::FILE_MAP_ALL_ACCESS;

#[macro_use]
extern crate log;

fn main() -> anyhow::Result<()> {
  std::env::set_var("RUST_LOG", "debug");
  pretty_env_logger::init();

  info!("hello awa");

  if let Ok(installation_path) = mahm::installation_path() {
    info!("MSI Afterburner is installed at {}", installation_path);
  } else {
    warn!("Could not find MSI Afterburner installation path");
  }

  let shared_memory = SharedMemory::new().context("failed to init shared memory")?;

  let header = shared_memory.header()?;

  debug_header(header);

  Ok(())
}

pub struct SharedMemory<'a> {
  pub handle: FileMapping<'a>,
  pub data: MemoryMappedView<'a>,
}

impl<'a> SharedMemory<'a> {
  fn new() -> anyhow::Result<Self> {
    let handle =
      open_file_mapping(FILE_MAP_ALL_ACCESS, false, "MAHMSharedMemory").context("failed to open shared memory")?;
    let data = map_view_of_file(handle.clone(), FILE_MAP_ALL_ACCESS, 0, 0).context("failed to map view of memory")?;

    Ok(Self { handle, data })
  }

  pub fn header(&'a self) -> Result<&'a MAHM_SHARED_MEMORY_HEADER, io::Error> {
    let header = unsafe { &*(self.data.0 as *const MAHM_SHARED_MEMORY_HEADER) };
    const EXPECTED_SIGNATURE: u32 = u32::from_be_bytes([b'M', b'A', b'H', b'M']);

    if header.dwSignature != EXPECTED_SIGNATURE {
      Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("unexpected signature: 0x{:X}", header.dwSignature),
      ))
    } else {
      Ok(header)
    }
  }
}

fn debug_header(header: &MAHM_SHARED_MEMORY_HEADER) {
  debug!("dwSignature: 0x{:X}", header.dwSignature);
  debug!("dwVersion: {}", header.dwVersion);
  debug!("dwHeaderSize: {}", header.dwHeaderSize);
  debug!("dwNumEntries: {}", header.dwNumEntries);
  debug!("dwEntrySize: {}", header.dwEntrySize);
  debug!("time: {}", header.time);
  debug!("dwNumGpuEntries: {}", header.dwNumGpuEntries);
  debug!("dwGpuEntrySize: {}", header.dwGpuEntrySize);
}
