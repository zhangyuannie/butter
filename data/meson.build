data_conf = configuration_data()
data_conf.set('LIBEXECDIR', libexecdir)
data_conf.set('BINDIR', bindir)

configure_file(
  input: '@0@.policy.in'.format(id),
  output: '@0@.policy'.format(id),
  configuration: data_conf,
  install: true,
  install_dir: datadir / 'polkit-1' / 'actions',
)

systemd = dependency('systemd')
systemd_unit_dir = systemd.get_variable(pkgconfig: 'systemdsystemunitdir')

configure_file(
  input: 'butter-schedule-snapshot.service.in',
  output: 'butter-schedule-snapshot.service',
  configuration: data_conf,
  install: true,
  install_dir: systemd_unit_dir,
)

configure_file(
  input: 'butter-schedule-prune.service.in',
  output: 'butter-schedule-prune.service',
  configuration: data_conf,
  install: true,
  install_dir: systemd_unit_dir,
)

install_data(
  'butter-schedule-snapshot.timer',
  install_dir: systemd_unit_dir
)

install_data(
  'butter-schedule-prune.timer',
  install_dir: systemd_unit_dir
)

meson.add_install_script('restorecon',
  systemd_unit_dir / 'butter-schedule-snapshot.service',
  systemd_unit_dir / 'butter-schedule-prune.service',
  systemd_unit_dir / 'butter-schedule-snapshot.timer',
  systemd_unit_dir / 'butter-schedule-prune.timer',
  bindir / 'butter',
)

install_data(
  '@0@.desktop'.format(id),
  install_dir: datadir / 'applications',
)

install_data(
  '@0@.gschema.xml'.format(id),
  install_dir: datadir / 'glib-2.0' / 'schemas',
)

subdir('icons')
subdir('resources')
