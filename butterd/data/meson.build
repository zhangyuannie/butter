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
