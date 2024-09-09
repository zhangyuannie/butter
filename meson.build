project(
  'butter',
  'rust',
  version: '1.1',
  meson_version: '>= 0.60',
)

i18n = import('i18n')
gnome = import('gnome')

id = 'org.zhangyuannie.Butter'

cargo = find_program('cargo', required: true)

script_restorecon = find_program('scripts/restorecon')
script_cargo_build = find_program('scripts/cargo_build')

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')
libexecdir = prefix / get_option('libexecdir')
datadir = prefix / get_option('datadir')
sharedstatedir = prefix / get_option('sharedstatedir')

pkgdatadir = datadir / meson.project_name()
pkgsharedstatedir = sharedstatedir / meson.project_name()
pkgsysconfdir = get_option('sysconfdir') / meson.project_name()

subdir('data')
subdir('po')
subdir('src')

install_emptydir(pkgsysconfdir / 'schedules')

gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: true,
  update_desktop_database: true,
)
