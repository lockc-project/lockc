subdir('bpf')

lockc_sources = files(
  'bin/lockcd.rs',
  'bin/lockc-runc-wrapper.rs',
  'bpf/lockc.skel.rs',
  'bpf/mod.rs',
  'lib.rs',
  'settings.rs',
)

cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: [
    cargo_manifests,
    lockc_sources,
  ],
  depends: [
    bpf_skeleton,
  ],
  output: [
    'lockcd',
    'lockc-runc-wrapper',
  ],
  install: true,
  install_dir: bindir,
  console: true,
  command: [
    cargo_script,
    '@SOURCE_ROOT@',
    '@OUTPUT0@',
    '@OUTPUT1@',
    meson.build_root(),
  ]
)
