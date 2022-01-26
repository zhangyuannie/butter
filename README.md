# Butter

Butter is a simple GTK application for BTRFS snapshot management.

## Build & Install

Dependencies:

- cargo
- libadwaita >= 1.0.0
- gtk >= 4.4.0
- btrfs-progs >= 5.0.0

On Fedora Linux 35+, you can install the required packages with:
```
# dnf install gtk4-devel libadwaita-devel cargo btrfs-progs
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
