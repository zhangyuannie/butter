conf = configuration_data()
conf.set_quoted('ID', id)
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', meson.project_name())
conf.set_quoted('LOCALEDIR', localedir)
conf.set_quoted('LIBEXECDIR', libexecdir)
conf.set_quoted('PKGDATADIR', pkgdatadir)
conf.set_quoted('PKGSHAREDSTATEDIR', pkgsharedstatedir)

configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: conf
)

run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

cargo_options = [
  '--manifest-path', meson.project_source_root() / 'Cargo.toml',
  '--target-dir', meson.project_build_root() / 'src',
  '--release',
]

cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / '.cargo' ]

cargo_build = custom_target(
  'cargo_build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  command: [
    'env', cargo_env,
    cargo, 'build', cargo_options,
    '&&',
    'cp', 'src' / 'release' / meson.project_name(), '@OUTPUT@',
  ],
)

subdir('daemon')
