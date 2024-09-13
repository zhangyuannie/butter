conf = configuration_data()
conf.set_quoted('ID', id)
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', meson.project_name())
conf.set_quoted('LOCALEDIR', localedir)
conf.set_quoted('LIBEXECDIR', libexecdir)
conf.set_quoted('PKGDATADIR', pkgdatadir)
conf.set_quoted('PKGSHAREDSTATEDIR', pkgsharedstatedir)
conf.set_quoted('PKGSYSCONFDIR', pkgsysconfdir)

configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: conf
)

run_command(
  'cp',
  meson.project_build_root() / 'butterd/src/config.rs',
  meson.project_source_root() / 'butterd/src/config.rs',
  check: true
)

custom_target(
  'cargo_build_butterd',
  build_by_default: true,
  build_always_stale: true,
  output: ['butterd'],
  console: true,
  install: true,
  install_dir: [libexecdir],
  command: [
    script_cargo_build,
    '--source-root', '@SOURCE_ROOT@',
    '--build-root', '@BUILD_ROOT@',
    '--package', 'butterd',
    '--output', '@OUTPUT@',
  ],
)
