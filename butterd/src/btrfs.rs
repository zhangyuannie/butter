use std::{
    collections::HashMap,
    ffi::CStr,
    fs, io,
    mem::{self, MaybeUninit},
    os::{
        raw::c_char,
        unix::prelude::{AsFd, AsRawFd},
    },
    ptr::addr_of_mut,
};

use nix::errno::Errno;
use uuid::Uuid;

use crate::{
    interface::BtrfsFilesystem,
    ioctl::{self, BtrfsDevInfoArgs, BtrfsFsInfoArgs, BTRFS_LABEL_SIZE},
    mnt_entry::MntEntries,
};

fn fs_info<T: AsFd>(fd: T) -> io::Result<(BtrfsFsInfoArgs, Vec<BtrfsDevInfoArgs>)> {
    let mut ret = MaybeUninit::<BtrfsFsInfoArgs>::uninit();
    let fd = fd.as_fd();
    unsafe { ioctl::btrfs_fs_info(fd.as_raw_fd(), ret.as_mut_ptr())? };
    let fs_info = unsafe { ret.assume_init() };
    let mut dev_vec = Vec::with_capacity(fs_info.num_devices as usize);
    for i in 0..=fs_info.max_id {
        if dev_vec.len() == fs_info.num_devices as usize {
            break;
        }
        match device_info(fd, i) {
            Ok(dev_info) => dev_vec.push(dev_info),
            Err(e) => {
                if e == Errno::ENODEV {
                    continue;
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    Ok((fs_info, dev_vec))
}

fn device_info<T: AsFd>(fd: T, device_id: u64) -> nix::Result<BtrfsDevInfoArgs> {
    let mut ret = MaybeUninit::<BtrfsDevInfoArgs>::uninit();
    let ptr = ret.as_mut_ptr();

    unsafe {
        addr_of_mut!((*ptr).devid).write(device_id);
        addr_of_mut!((*ptr).uuid).write(mem::zeroed());
    }

    unsafe { ioctl::btrfs_dev_info(fd.as_fd().as_raw_fd(), ret.as_mut_ptr())? };

    Ok(unsafe { ret.assume_init() })
}

fn fs_label_from_mounted<T: AsFd>(fd: T) -> nix::Result<String> {
    let mut out = MaybeUninit::<[c_char; BTRFS_LABEL_SIZE]>::uninit();
    unsafe {
        ioctl::btrfs_get_fslabel(fd.as_fd().as_raw_fd(), out.as_mut_ptr())?;
    }
    let out = unsafe { out.assume_init() };
    let ret = unsafe { CStr::from_ptr(out.as_ptr()) };
    Ok(ret.to_string_lossy().to_string())
}

/// Get all mounted Btrfs filesystems.
/// This probably has a lot of edge case such as seed devices and what's not
pub fn read_all_mounted_btrfs_fs() -> io::Result<Vec<BtrfsFilesystem>> {
    let f = fs::File::open("/proc/self/mounts")?;
    let entries = MntEntries::new(io::BufReader::new(f));

    let mut ret = HashMap::<Uuid, BtrfsFilesystem>::new();

    for entry in entries {
        if let Ok(entry) = entry {
            if entry.fs_type != "btrfs" {
                continue;
            }
            let mnt_path = &entry.target.ok_or(io::ErrorKind::InvalidData)?;
            let file = fs::File::open(mnt_path)?;

            let (fs_info, dev_infos) = fs_info(&file)?;

            let fs_uuid = Uuid::from_bytes(fs_info.fsid);
            if ret.contains_key(&fs_uuid) {
                continue;
            }

            ret.insert(
                fs_uuid,
                BtrfsFilesystem {
                    label: fs_label_from_mounted(&file)?,
                    uuid: fs_uuid,
                    devices: dev_infos
                        .iter()
                        .map(|dev| unsafe {
                            CStr::from_ptr(dev.path.as_ptr() as *const c_char)
                                .to_string_lossy()
                                .to_string()
                        })
                        .collect(),
                },
            );
        }
    }
    Ok(ret.into_values().collect())
}
