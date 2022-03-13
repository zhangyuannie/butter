import unittest
from unittest.mock import patch
import main


class TestFindVolMnt(unittest.TestCase):
    def test_fedora_layout(self):
        layout = '{"filesystems":[{"target":"/","source":"/dev/nvme0n1p2[/root]","fstype":"btrfs","options":"rw,relatime,seclabel,compress=zstd:1,ssd,space_cache,subvolid=257,subvol=/root","children":[{"target":"/home","source":"/dev/nvme0n1p2[/home]","fstype":"btrfs","options":"rw,relatime,seclabel,compress=zstd:1,ssd,space_cache,subvolid=256,subvol=/home"},{"target":"/boot","source":"/dev/nvme0n1p2[/boot]","fstype":"btrfs","options":"rw,relatime,seclabel,compress=zstd:1,ssd,space_cache,subvolid=258,subvol=/boot"}]}]}'

        with patch("main.subprocess.check_output") as check_output:
            check_output.return_value = layout
            ret = main.find_subvol_mnt()
        self.assertEqual(ret, {"/root": "/", "/boot": "/boot", "/home": "/home"})

    def test_opensuse_layout(self):
        layout = '{"filesystems":[{"target":"/","source":"/dev/vda2[/@/.snapshots/1/snapshot]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=266,subvol=/@/.snapshots/1/snapshot","children":[{"target":"/.snapshots","source":"/dev/vda2[/@/.snapshots]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=265,subvol=/@/.snapshots"},{"target":"/boot/grub2/i386-pc","source":"/dev/vda2[/@/boot/grub2/i386-pc]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=264,subvol=/@/boot/grub2/i386-pc"},{"target":"/boot/grub2/x86_64-efi","source":"/dev/vda2[/@/boot/grub2/x86_64-efi]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=263,subvol=/@/boot/grub2/x86_64-efi"},{"target":"/home","source":"/dev/vda2[/@/home]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=262,subvol=/@/home"},{"target":"/opt","source":"/dev/vda2[/@/opt]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=261,subvol=/@/opt"},{"target":"/root","source":"/dev/vda2[/@/root]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=260,subvol=/@/root"},{"target":"/srv","source":"/dev/vda2[/@/srv]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=259,subvol=/@/srv"},{"target":"/usr/local","source":"/dev/vda2[/@/usr/local]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=258,subvol=/@/usr/local"},{"target":"/var","source":"/dev/vda2[/@/var]","fstype":"btrfs","options":"rw,relatime,space_cache=v2,subvolid=257,subvol=/@/var"}]}]}'

        with patch("main.subprocess.check_output") as check_output:
            check_output.return_value = layout
            ret = main.find_subvol_mnt()
        self.assertEqual(
            ret,
            {
                "/@/.snapshots/1/snapshot": "/",
                "/@/var": "/var",
                "/@/usr/local": "/usr/local",
                "/@/srv": "/srv",
                "/@/root": "/root",
                "/@/opt": "/opt",
                "/@/home": "/home",
                "/@/boot/grub2/x86_64-efi": "/boot/grub2/x86_64-efi",
                "/@/boot/grub2/i386-pc": "/boot/grub2/i386-pc",
                "/@/.snapshots": "/.snapshots",
            },
        )


if __name__ == "__main__":
    unittest.main()
