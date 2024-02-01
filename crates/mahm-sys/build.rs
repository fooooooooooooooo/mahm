use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use bindgen::Builder;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

const ALLOWED_TYPES: [&str; 3] = ["MAHM.*", "DWORD", "__time32_t"];

fn main() -> Result<(), anyhow::Error> {
  let mut builder = Builder::default().clang_arg("-v");

  for path in find_windows_sdk_paths() {
    builder = builder.clang_arg(format!("-I{}", path.to_str().unwrap()));
  }

  let msvc_path = find_msvc_path().context("unable to find msvc path")?;
  builder = builder.clang_arg(format!("-I{}", msvc_path.to_str().unwrap()));

  for ty in ALLOWED_TYPES.iter() {
    builder = builder.allowlist_type(ty);
  }

  builder = builder
    .header("wrapper.h")
    .allowlist_recursively(false)
    .allowlist_file("wrapper.h")
    .allowlist_file("MAHMSharedMemory.h")
    .allowlist_function("MAHM.*")
    .allowlist_var("MAHM.*")
    .allowlist_var("MONITORING_SOURCE_.*");

  let bindings = builder
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .generate()
    .context("unable to generate bindings")?;

  let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("src/generated.rs"))
    .context("failed to write bindings")?;

  Ok(())
}

const HKLM: RegKey = RegKey::predef(HKEY_LOCAL_MACHINE);
const REGISTRY_PATH: &str = "SOFTWARE\\WOW6432Node\\Microsoft\\Microsoft SDKs\\Windows\\v10.0";
const SDK_INCLUDE_DIRS: [&str; 5] = ["um", "shared", "winrt", "ucrt", "cppwinrt"];

fn find_windows_sdk_paths() -> Vec<PathBuf> {
  let mut paths = Vec::new();

  let key = HKLM
    .open_subkey(REGISTRY_PATH)
    .expect("unable to read HKLM:SOFTWARE\\WOW6432Node\\Microsoft\\Microsoft SDKs\\Windows\\v10.0");

  let path: String = key
    .get_value("InstallationFolder")
    .expect("unable to get windows sdk installation folder");
  let version: String = key
    .get_value("ProductVersion")
    .expect("unable to get windows sdk version");

  let base = PathBuf::from(path).join("Include").join(format!("{version}.0"));

  for dir in SDK_INCLUDE_DIRS.iter() {
    paths.push(base.join(dir));
  }

  paths
}

const VSWHERE_PATH: &str = "C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe";
const VC_REDIST_VERSION_FILE: &str = "VC\\Auxiliary\\Build\\Microsoft.VCRedistVersion.default.txt";

fn find_msvc_path() -> anyhow::Result<PathBuf> {
  // run `vswhere.exe -latest -property installationPath` to find the latest
  // installed visual studio

  let result = Command::new(VSWHERE_PATH)
    .arg("-latest")
    .arg("-property")
    .arg("installationPath")
    .output()
    .context("unable to run vswhere.exe")?;

  if !result.status.success() {
    return Err(anyhow::anyhow!("vswhere.exe failed with exit code {}", result.status));
  }

  let path = String::from_utf8(result.stdout).context("vswhere.exe returned invalid utf8")?;
  let path = PathBuf::from(path.trim());

  let vc_redist_version_path = path.join(VC_REDIST_VERSION_FILE);
  let version = read_to_string(vc_redist_version_path.clone()).context(format!(
    "unable to read VCRedistVersion file at {vc_redist_version_path:?}"
  ))?;
  let version = version.trim();

  Ok(path.join("VC\\Tools\\MSVC").join(version).join("include"))
}
