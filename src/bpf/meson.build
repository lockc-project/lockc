btf = custom_target(
  'btf',
  build_by_default: true,
  output: [
    'vmlinux.h',
  ],
  install: false,
  console: true,
  command: [
    btf_script,
    '@SOURCE_ROOT@',
  ]
)

bpf_sources = files(
  'lockc.bpf.c',
)

bpf_skeleton = custom_target(
  'cargo-libbpf',
  build_by_default: true,
  input: [
    bpf_sources,
  ],
  depends: [
    btf,
  ],
  output: [
    'lockc.skel.rs',
  ],
  install: false,
  console: true,
  command: [
    libbpf_script,
    meson.get_compiler('c').get_id(),
  ]
)
