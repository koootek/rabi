#[cfg(feature = "derive")]
pub mod derive {
    pub use rabi_derive::{FromRaw, IntoRaw};
}

pub use rabi_core::*;
