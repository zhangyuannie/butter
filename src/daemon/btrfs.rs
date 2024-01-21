use std::{
    collections::HashMap,
    ffi::{CStr, OsStr},
    fs, io,
    mem::{self, MaybeUninit},
    os::{
        raw::c_char,
        unix::prelude::{AsFd, AsRawFd, OsStrExt},
    },
    path::{Path, PathBuf},
    ptr::addr_of_mut,
};

use nix::errno::Errno;
use uuid::Uuid;

use crate::{filesystem::Filesystem, subvolume::SnapshotMetadata, write_as_json};

use super::{
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
    let mut out: [c_char; BTRFS_LABEL_SIZE] = [0; BTRFS_LABEL_SIZE];
    unsafe {
        ioctl::btrfs_get_fslabel(fd.as_fd().as_raw_fd(), &mut out)?;
    }
    let ret = unsafe { CStr::from_ptr(out.as_ptr()) };
    Ok(ret.to_string_lossy().to_string())
}

fn subvol_id_from_mnt_options(options: &str) -> Option<u64> {
    for seg in options.split(',') {
        if seg.starts_with("subvolid=") {
            return seg[9..].parse::<u64>().ok();
        }
    }
    None
}

/// Get all mounted Btrfs filesystems.
/// This probably has a lot of edge case such as seed devices and what's not
pub fn read_all_mounted_btrfs_fs() -> io::Result<Vec<Filesystem>> {
    let f = fs::File::open("/proc/self/mounts")?;
    let entries = MntEntries::new(io::BufReader::new(f));

    let mut ret = HashMap::<Uuid, Filesystem>::new();

    for entry in entries {
        if let Ok(entry) = entry {
            if entry.fs_type != "btrfs" {
                continue;
            }
            let mnt_path = &entry.target.ok_or(io::ErrorKind::InvalidData)?;
            let file = fs::File::open(mnt_path)?;

            let (fs_info, dev_infos) = fs_info(&file)?;

            let fs_uuid = Uuid::from_bytes(fs_info.fsid);

            let subvol_id =
                subvol_id_from_mnt_options(&entry.options).ok_or(io::ErrorKind::InvalidData)?;

            if ret.contains_key(&fs_uuid) {
                ret.get_mut(&fs_uuid)
                    .unwrap()
                    .mounts
                    .entry(subvol_id)
                    .or_insert(Vec::new())
                    .push(mnt_path.to_owned());
                continue;
            }

            ret.insert(
                fs_uuid,
                Filesystem {
                    label: fs_label_from_mounted(&file)?,
                    uuid: fs_uuid,
                    devices: dev_infos
                        .iter()
                        .map(|dev| unsafe {
                            PathBuf::from(OsStr::from_bytes(
                                CStr::from_ptr(dev.path.as_ptr() as *const c_char).to_bytes(),
                            ))
                        })
                        .collect(),
                    mounts: HashMap::from([(subvol_id, vec![mnt_path.to_owned()])]),
                },
            );
        }
    }
    Ok(ret.into_values().collect())
}

/// Create a regular snapshot, save butter specific metadata, conditionally make it read-only
pub fn create_butter_snapshot(src_mnt: &Path, dst_mnt: &Path, readonly: bool) -> io::Result<()> {
    let src_subvol_path = libbtrfsutil::subvolume_path(src_mnt).or_else(|e| Err(e.os_error()))?;
    if let Some(dst_parent) = dst_mnt.parent() {
        fs::create_dir_all(dst_parent)?;
    }
    libbtrfsutil::CreateSnapshotOptions::new()
        .create(src_mnt, dst_mnt)
        .or_else(|e| Err(e.os_error()))?;
    let metadata_dir = dst_mnt.join(".butter");
    fs::create_dir_all(&metadata_dir)?;

    let metadata = SnapshotMetadata {
        created_from: src_subvol_path,
        uuid: libbtrfsutil::subvolume_info(dst_mnt)
            .or_else(|e| Err(e.os_error()))?
            .uuid(),
    };
    write_as_json(&metadata_dir.join("info.json"), &metadata)?;

    libbtrfsutil::set_subvolume_read_only(&dst_mnt, readonly).or_else(|e| Err(e.os_error()))?;
    Ok(())
}
