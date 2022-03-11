project(
  'butter',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.59',
)

i18n = import('i18n')
gnome = import('gnome')

id = 'org.zhangyuannie.butter'

cargo = find_program('cargo', required: true)

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')
libexecdir = prefix / get_option('libexecdir')
datadir = prefix / get_option('datadir')

pkgdatadir = datadir / meson.project_name()

subdir('data')
subdir('po')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
