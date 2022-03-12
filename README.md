# Butter

Butter is a simple GTK application for BTRFS snapshot management.

> WIP. DO NOT USE.

![Screenshot](https://znie.org/images/butter/screenshot.png)

## Build & Install

Dependencies:

- cargo
- libadwaita >= 1.0.0
- gtk >= 4.4.0
- python3-btrfsutil >= 5.10.0
- kernel >= 4.18
- polkit >= 0.100
- meson >= 0.59
- python >= 3.6

You can install the required packages with:

- Fedora Linux 35 and newer

  ```
  # dnf install meson cargo gtk4-devel libadwaita-devel python3-btrfsutil
  ```

- openSUSE Tumbleweed

  ```
  # zypper install meson cargo gtk4-devel libadwaita-devel python-btrfsutil
  ```

To build and install:

```
$ meson --prefix=/usr build
$ ninja -C build
# ninja -C build install
```

To uninstall:

```
# ninja -C build uninstall
```
