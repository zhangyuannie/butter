policy_conf = configuration_data()
policy_conf.set('LIBEXECDIR', libexecdir)

configure_file(
  input: '@0@.policy.in'.format(id),
  output: '@0@.policy'.format(id),
  configuration: policy_conf,
  install: true,
  install_dir: datadir / 'polkit-1' / 'actions',
)

install_data(
  '@0@.desktop'.format(id),
  install_dir: datadir / 'applications',
)

subdir('icons')
subdir('resources')