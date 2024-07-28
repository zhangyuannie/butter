# Building Butter

## Dependencies

- cargo (build-time only)
- clang (build-time only)
- meson >= 0.59 (build-time only)
- python 3 (build-time only)
- libadwaita >= 1.2.0
- gtk >= 4.8.0
- libbtrfsutil >= 1.2.0
- kernel >= 4.18
- polkit
- systemd (timer)

You can install these dependencies with:

- Arch

  ```
  # pacman -S meson rust gtk4 libadwaita btrfs-progs clang
  ```

- Fedora

  ```
  # dnf install meson cargo gtk4-devel libadwaita-devel btrfs-progs-devel clang
  ```

- openSUSE Tumbleweed

  ```
  # zypper install meson cargo gtk4-devel libadwaita-devel libbtrfsutil-devel llvm-clang
  ```

## Get the Source Code

```
$ git clone https://github.com/zhangyuannie/butter.git
$ cd butter
```

## Build and Install

To install Butter into `/usr`:

```
$ meson --prefix=/usr build
$ ninja -C build
# ninja -C build install
# systemctl reload dbus
```

To uninstall:

```
# ninja -C build uninstall
```
