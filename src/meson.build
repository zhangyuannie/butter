custom_target(
  'cargo_build_butter',
  build_by_default: true,
  build_always_stale: true,
  output: ['butter'],
  console: true,
  install: true,
  install_dir: [bindir],
  command: [
    script_cargo_build,
    '--source-root', '@SOURCE_ROOT@',
    '--build-root', '@BUILD_ROOT@',
    '--package', 'butter',
    '--output', '@OUTPUT@',
  ],
)
