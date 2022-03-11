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

On Fedora Linux 35+, you can install the required packages with:
```
# dnf install gtk4-devel libadwaita-devel cargo python3-btrfsutil meson
```

To build and install:
```
$ meson --prefix=/usr build
$ ninja -C build
$ sudo ninja -C build install
```

To uninstall:
```
# sudo ninja -C build uninstall
```

## Contributing

Unfortunately, this project is closed to contributions right now.
