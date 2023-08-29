use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{bail, Context};
use libloading::Library;

use crate::internal_shared::cargo_path_utils;

struct LibraryHolderInner(Option<Library>, PathBuf);

impl Drop for LibraryHolderInner {
    fn drop(&mut self) {
        self.0 = None;
        let _ = std::fs::remove_file(&self.1);
    }
}

impl LibraryHolderInner {
    pub fn new(path: &PathBuf) -> Option<Self> {
        let extension = path.extension();
        let uuid = uuid::Uuid::new_v4();
        let new_path = path.clone();
        let mut new_path = new_path.with_file_name(uuid.to_string());
        if let Some(extension) = extension {
            new_path.set_extension(extension);
        }
        crate::logger::debug!("New path: {new_path:?}");
        std::fs::rename(path, &new_path).ok()?;
        crate::logger::debug!("Copied file to new path");

        await_file(3, &new_path);

        let _folder = new_path.parent();

        crate::logger::debug!("Search Paths: ");
        for path in cargo_path_utils::dylib_path() {
            crate::logger::debug!("{path:?}");
            if let Ok(dir) = std::fs::read_dir(&path) {
                for entry in dir.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains("libbevy_dylib") {
                        crate::logger::debug!("Found bevy dylib at {:?}", entry.path());
                    }
                }
            } else {
                crate::logger::error!("THIS PATH DOES NOT EXIST - {path:?}");
            }
        }

        // SAFETY: Here we are relying on libloading's safety processes for ensuring the Library we receive is properly set up. We expect that library to respect rust ownership semantics because we control it's compilation and know that it is built in rust as well, but the wrappers are unaware so they rely on unsafe.
        match unsafe { libloading::Library::new(&new_path) } {
            Ok(lib) => {
                crate::logger::info!("Loaded library");
                Some(Self(Some(lib), new_path))
            }
            Err(err) => {
                crate::logger::error!("Error loading library: {err:?}");

                crate::logger::error!("Search Paths: ");
                for path in cargo_path_utils::dylib_path() {
                    crate::logger::error!("{path:?}");
                }

                None
            }
        }
    }

    pub fn library(&self) -> Option<&Library> {
        self.0.as_ref()
    }

    pub fn call<T>(&self, name: &str, args: &mut T) -> anyhow::Result<()> {
        let Some(lib) = &self.0 else {
            bail!("Library Unavailable")
        };

        crate::logger::debug!("Preparing to call {name}");

        // SAFETY: This should be safe due to relying on rust ownership semantics for passing values between two rust crates. Since we know that the library itself is a rust rather than C library, we know that it will respect a mutable borrow internally.
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(&mut T)> =
                lib.get(name.as_bytes())
                    .context(format!("Couldn't load function {name}"))?;
            crate::logger::debug!("Got symbol");
            func(args);
            crate::logger::debug!("Call complete");
        };
        Ok(())
    }
}

fn await_file(iterations: usize, path: &PathBuf) {
    if path.exists() {
        crate::logger::debug!("Validated {path:?} Exists");
        return;
    }
    if iterations > 0 {
        crate::logger::debug!("{path:?} doesn't exist yet...");
        await_file(iterations.saturating_sub(1), path);
        std::thread::sleep(Duration::from_secs_f32(0.5));
    }
}

#[derive(Clone)]
pub struct LibraryHolder(Arc<LibraryHolderInner>);

impl LibraryHolder {
    pub fn new(path: &PathBuf) -> Option<Self> {
        let inner = LibraryHolderInner::new(path)?;
        Some(Self(Arc::new(inner)))
    }
    pub fn library(&self) -> Option<&Library> {
        self.0.library()
    }

    pub fn call<T>(&self, name: &str, args: &mut T) -> anyhow::Result<()> {
        self.0.call(name, args)
    }
}