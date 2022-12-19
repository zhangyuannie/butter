use std::os::raw::c_char;

use nix::{ioctl_read, ioctl_readwrite};

const BTRFS_IOCTL_MAGIC: u8 = 0x94;
const BTRFS_DEVICE_PATH_NAME_MAX: usize = 1024;
pub const BTRFS_LABEL_SIZE: usize = 256;

/// btrfs_ioctl_fs_info_args from /include/uapi/linux/btrfs.h
#[repr(C)]
#[derive(Clone, Debug)]
pub struct BtrfsFsInfoArgs {
    pub max_id: u64,
    pub num_devices: u64,
    pub fsid: [u8; 16],
    pub nodesize: u32,
    pub sectorsize: u32,
    pub clone_alignment: u32,
    pub csum_type: u16,
    pub csum_size: u16,
    /// in/out
    pub flags: u64,
    pub generation: u64,
    pub metadata_uuid: [u8; 16],
    reserved: [u8; 944],
}

/// btrfs_ioctl_dev_info_args from /include/uapi/linux/btrfs.h
#[repr(C)]
#[derive(Clone, Debug)]
pub struct BtrfsDevInfoArgs {
    /// in/out
    pub devid: u64,
    /// in/out
    pub uuid: [u8; 16],
    pub bytes_used: u64,
    pub total_bytes: u64,
    unused: [u64; 379],
    pub path: [u8; BTRFS_DEVICE_PATH_NAME_MAX],
}

ioctl_read!(btrfs_fs_info, BTRFS_IOCTL_MAGIC, 31, BtrfsFsInfoArgs);
ioctl_readwrite!(btrfs_dev_info, BTRFS_IOCTL_MAGIC, 30, BtrfsDevInfoArgs);
ioctl_read!(
    btrfs_get_fslabel,
    BTRFS_IOCTL_MAGIC,
    49,
    [c_char; BTRFS_LABEL_SIZE]
);
