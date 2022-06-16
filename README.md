# Butter

Butter is a simple GTK application for Btrfs snapshot management.

> Work in Progress.

![Screenshot](https://znie.org/images/butter/screenshot.png)

## Installation

### Distribution repositories

Development versions tracking the main branch:

- Arch: https://aur.archlinux.org/packages/butter-git
- Fedora: https://copr.fedorainfracloud.org/coprs/zhangyuannie/butter-git

### Building from Source

#### Dependencies

- cargo
- libadwaita >= 1.0.0
- gtk >= 4.4.0
- btrfs-progs >= 5.10.0
- libbtrfsutil >= 5.10.0
- kernel >= 4.18
- polkit >= 0.100
- meson >= 0.59

You can install these dependencies with:

- Arch
  ```
  # pacman -S meson rust gtk4 libadwaita btrfs-progs
  ```

- Fedora 35 and later

  ```
  # dnf install meson cargo gtk4-devel libadwaita-devel btrfs-progs libbtrfsutil
  ```

- openSUSE Tumbleweed

  ```
  # zypper install meson cargo gtk4-devel libadwaita-devel btrfsprogs libbtrfsutil1
  ```

#### Get the Source Code

```
$ git clone https://github.com/zhangyuannie/butter.git
$ cd butter
```

#### Build and Install

To install Butter into `/usr`:

```
$ meson --prefix=/usr build
$ ninja -C build
# ninja -C build install
```

To uninstall:

```
# ninja -C build uninstall
```

## License

Butter is available under the GNU General Public License version 3 (GPLv3). See [COPYING](COPYING) for the full text.
