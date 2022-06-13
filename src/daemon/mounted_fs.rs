use std::ffi::CString;
use std::{fs, path::Path};

use anyhow::{Context, Result};
use libc::c_void;
use tempfile::{tempdir_in, TempDir};

use super::libc::{mount, umount2, ToCString};

/// A Btrfs filesystem that is mounted
pub struct MountedTopLevelSubvolume {
    dir: TempDir,
}

impl MountedTopLevelSubvolume {
    pub fn new<T: AsRef<Path>>(device: T) -> Result<MountedTopLevelSubvolume> {
        fs::create_dir_all("/run/butter")?;
        let dir = tempdir_in("/run/butter")?;
        let data = CString::new("subvol=/").unwrap();

        mount(
            device.as_ref().to_cstring().unwrap(),
            dir.path().to_cstring().unwrap(),
            CString::new("btrfs").unwrap(),
            0,
            data.as_ptr() as *const c_void,
        )
        .context("failed to mount top level subvolume")?;

        Ok(MountedTopLevelSubvolume { dir })
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

impl Drop for MountedTopLevelSubvolume {
    fn drop(&mut self) {
        let _ = umount2(self.dir.path().to_cstring().unwrap(), libc::MNT_DETACH);
    }
}
