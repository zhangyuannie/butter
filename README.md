# Butter

Butter is a simple GTK application for Btrfs snapshot management.

> Work in Progress.

![Screenshot](https://znie.org/images/butter/screenshot.png)

## Installation

### Distribution repositories

Development versions tracking the main branch:

- Fedora: https://copr.fedorainfracloud.org/coprs/zhangyuannie/butter-git

### Building from Source

#### Dependencies

- cargo
- libadwaita >= 1.0.0
- gtk >= 4.4.0
- python3-btrfsutil >= 5.10.0
- kernel >= 4.18
- polkit >= 0.100
- meson >= 0.59
- python >= 3.6

You can install these dependencies with:

- Fedora 35 and later

  ```
  # dnf install meson cargo gtk4-devel libadwaita-devel python3-btrfsutil
  ```

- openSUSE Tumbleweed

  ```
  # zypper install meson cargo gtk4-devel libadwaita-devel python-btrfsutil
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
