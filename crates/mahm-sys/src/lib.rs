#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

pub mod bindings {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/generated.rs"));
}
