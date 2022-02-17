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

On Fedora Linux 35+, you can install the required packages with:
```
# dnf install gtk4-devel libadwaita-devel cargo python3-btrfsutil
```

To build and install:
```
$ make
# make install
```

To uninstall:
```
# make uninstall
```

## Contributing

Unfortunately, this project is closed to contributions right now.
