use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

#[macro_use]
extern crate log;

pub mod windows;

const HKLM: RegKey = RegKey::predef(HKEY_LOCAL_MACHINE);

fn msi_afterburner_regkey() -> std::io::Result<RegKey> {
  HKLM.open_subkey("Software\\MSI\\Afterburner")
}

pub fn installation_path() -> std::io::Result<String> {
  msi_afterburner_regkey().and_then(|regkey| regkey.get_value("InstallPath"))
}
