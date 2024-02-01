use std::ffi::OsStr;
use std::marker::PhantomData;
use std::os::windows::ffi::OsStrExt;
use std::rc::Rc;

use windows::core::{Error, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Memory::{
  MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP, MEMORY_MAPPED_VIEW_ADDRESS,
};

fn pcwstr(s: &str) -> PCWSTR {
  let v: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();

  PCWSTR::from_raw(v.as_ptr())
}

#[derive(Debug, Clone)]
pub struct FileMapping<'a>(pub Rc<HANDLE>, PhantomData<&'a ()>);

impl FileMapping<'_> {
  pub fn handle(&self) -> HANDLE {
    *self.0
  }

  pub fn id(&self) -> isize {
    self.handle().0
  }
}

impl<'a> Drop for FileMapping<'a> {
  fn drop(&mut self) {
    if Rc::strong_count(&self.0) > 1 {
      return;
    }

    trace!("closing handle({:?})", self.id());

    if let Err(e) = close_handle(self.handle()) {
      error!("failed to close handle({:?}): {:?}", self.id(), e);
    };
  }
}

pub fn open_file_mapping<'a>(access: FILE_MAP, inherit: bool, name: &str) -> Result<FileMapping<'a>, Error> {
  let lpname = pcwstr(name);

  let handle = unsafe { OpenFileMappingW(access.0, inherit, lpname) }?;

  trace!("opened file mapping: {:?}", handle.0);

  Ok(FileMapping(Rc::new(handle), PhantomData))
}

pub fn close_handle(handle: HANDLE) -> Result<(), Error> {
  unsafe { CloseHandle(handle) }
}

#[derive(Debug, Clone)]
pub struct MemoryMappedView<'a>(pub *mut u8, PhantomData<&'a ()>);

impl<'a> Drop for MemoryMappedView<'a> {
  fn drop(&mut self) {
    if let Err(e) = unmap_view_of_file(self.0) {
      error!("failed to unmap view of file: {:?}", e);
    }
  }
}

pub fn map_view_of_file<'a>(
  handle: FileMapping,
  access: FILE_MAP,
  offset: u64,
  size: usize,
) -> Result<MemoryMappedView<'a>, Error> {
  let offset_high = (offset >> 32) as u32;
  let offset_low = offset as u32;

  let result = unsafe { MapViewOfFile(*handle.0, access, offset_high, offset_low, size) };

  if result.Value.is_null() {
    Err(Error::from_win32())
  } else {
    Ok(MemoryMappedView(result.Value as *mut u8, PhantomData))
  }
}

pub fn unmap_view_of_file(data: *mut u8) -> Result<(), Error> {
  unsafe { UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: data as _ }) }
}
