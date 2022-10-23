install_data(
  'data' / 'org.zhangyuannie.Butter1.conf',
  install_dir: datadir / 'dbus-1' / 'system.d',
)

configure_file(
  input: 'data/org.zhangyuannie.Butter1.service.in',
  output: 'org.zhangyuannie.Butter1.service',
  configuration: data_conf,
  install: true,
  install_dir: datadir / 'dbus-1' / 'system-services',
)

custom_target(
  'cargo_build_butterd',
  build_by_default: true,
  build_always_stale: true,
  output: 'butterd',
  console: true,
  install: true,
  install_dir: libexecdir,
  command: [
    script_cargo_build,
    '--source-root', '@SOURCE_ROOT@',
    '--build-root', '@BUILD_ROOT@',
    '--package', 'butterd',
    '--output', '@OUTPUT@',
  ],
)
