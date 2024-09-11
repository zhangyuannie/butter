data_conf = configuration_data()
data_conf.set('LIBEXECDIR', libexecdir)
data_conf.set('BINDIR', bindir)
data_conf.set('ID', id)

configure_file(
  input: '@0@.policy.in'.format(id.to_lower()),
  output: '@0@.policy'.format(id.to_lower()),
  configuration: data_conf,
  install: true,
  install_dir: datadir / 'polkit-1' / 'actions',
)

install_data(
  f'@id@.desktop',
  install_dir: datadir / 'applications',
)

install_data(
  f'@id@.gschema.xml',
  install_dir: datadir / 'glib-2.0' / 'schemas',
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

install_data(
  'org.zhangyuannie.Butter1.conf',
  install_dir: datadir / 'dbus-1' / 'system.d',
)

configure_file(
  input: 'org.zhangyuannie.Butter1.service.in',
  output: 'org.zhangyuannie.Butter1.service',
  configuration: data_conf,
  install: true,
  install_dir: datadir / 'dbus-1' / 'system-services',
)

meson.add_install_script(script_reseat_file,
  systemd_unit_dir / 'butter-schedule-snapshot.service',
  systemd_unit_dir / 'butter-schedule-prune.service',
  systemd_unit_dir / 'butter-schedule-snapshot.timer',
  systemd_unit_dir / 'butter-schedule-prune.timer',
  bindir / 'butter',
  datadir / 'dbus-1/system-services/org.zhangyuannie.Butter1.service',
  datadir / 'polkit-1/actions' / '@0@.policy'.format(id.to_lower()),
)

subdir('icons')
subdir('resources')
