# Buttercream

Buttercream is a simple GTK application for BTRFS snapshot management.

## Build Instructions

Dependencies:

- cargo
- libadwaita >= 1.0.0
- gtk >= 4.4.0

On Fedora Linux 35+, you can install the required packages with:
```
# dnf install gtk4-devel libadwaita-devel cargo
```
To build the binary:

```
$ cargo build --release
```
