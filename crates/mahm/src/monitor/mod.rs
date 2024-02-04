use mahm_sys::bindings::MAHM_SHARED_MEMORY_ENTRY;
use windows::Win32::System::Memory::FILE_MAP_ALL_ACCESS;

use crate::monitor::entry::Entry;
use crate::monitor::header::{Header, InvalidHeaderError};
use crate::windows::{map_view_of_file, open_file_mapping, FileMapping, MemoryMappedView};

pub mod entry;
pub mod header;

struct HardwareMonitorMemory<'a> {
  #[allow(dead_code)]
  handle: FileMapping<'a>,
  data: MemoryMappedView<'a>,
}

impl<'a> HardwareMonitorMemory<'a> {
  pub fn read<T>(&self) -> &'a T {
    unsafe { &*(self.data.0 as *const _) }
  }

  pub fn read_entry(&self, header: &'a Header, index: u32) -> &'a MAHM_SHARED_MEMORY_ENTRY {
    let offset = header.header_size + (header.entry_size * index);
    unsafe { &*(self.data.0.add(offset as usize) as *const _) }
  }
}

pub struct HardwareMonitor<'a> {
  memory: Option<HardwareMonitorMemory<'a>>,

  pub header: Option<Header<'a>>,
  pub entries: Vec<Entry<'a>>,
}

impl<'a> HardwareMonitor<'a> {
  pub fn new() -> Self {
    Self {
      memory: None,
      header: None,
      entries: vec![],
    }
  }

  pub fn refresh(&mut self) -> Result<(), RefreshError> {
    self.open_memory()?;

    let data = self.memory.as_ref().unwrap();

    let header = unsafe { Header::new(data.read()) }?;

    let entries = (0..header.entry_count)
      .map(|i| unsafe { Entry::new(data.read_entry(&header, i)) })
      .collect::<Vec<_>>();

    self.header = Some(header);
    self.entries = entries;

    Ok(())
  }

  fn close_memory(&mut self) {
    self.memory = None;
  }

  fn open_memory(&mut self) -> Result<(), windows::core::Error> {
    self.close_memory();

    let handle = open_file_mapping(FILE_MAP_ALL_ACCESS, false, "MAHMSharedMemory")?;
    let data = map_view_of_file(handle.clone(), FILE_MAP_ALL_ACCESS, 0, 0)?;

    self.memory = Some(HardwareMonitorMemory { handle, data });

    Ok(())
  }
}

impl<'a> Default for HardwareMonitor<'a> {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(thiserror::Error, Debug)]
pub enum RefreshError {
  #[error(transparent)]
  InvalidHeader(#[from] InvalidHeaderError),
  #[error(transparent)]
  WindowsError(#[from] windows::core::Error),
}
