project(
  'lockc', 'rust', 'c',
  version: '0.1.0',
  license: 'Apache-2.0 AND GPL-2.0-or-later',
)

cargo = find_program('cargo', required: true)

confdir = '/etc'
bindir = join_paths(get_option('prefix'), get_option('bindir'))
unitdir = join_paths(get_option('prefix'), 'lib/systemd/system')

btf_script = find_program('scripts/btf.sh')
libbpf_script = find_program('scripts/libbpf.sh')
cargo_script = find_program('scripts/cargo.sh')
test_script = find_program('scripts/test.sh')

cargo_manifests = files(
  'Cargo.toml',
)

subdir('src')

test(
  'cargo-test',
  test_script,
  args: meson.build_root(),
  workdir: meson.source_root(),
  timeout: 3000,
)

lockcd_systemd_unit = configure_file(
  input: 'contrib/systemd/lockcd.service.in',
  output: 'lockcd.service',
  configuration: {
    'PREFIX': get_option('prefix'),
  },
  install: true,
  install_dir: unitdir,
)

containerd_confdir = install_subdir(
  'containerd',
  install_dir: confdir,
)
containerd_confdir_path = join_paths(confdir, 'containerd')

containerd_config = configure_file(
  depends: [
    containerd_confdir,
  ],
  input: 'contrib/etc/containerd/config.toml.in',
  output: 'config.toml',
  configuration: {
    'BINDIR': bindir,
  },
  install: true,
  install_dir: containerd_confdir_path,
)

docker_confdir = install_subdir(
  'docker',
  install_dir: confdir,
)
docker_confdir_path = join_paths(confdir, 'docker')

docker_config = configure_file(
  depends: [
    docker_confdir,
  ],
  input: 'contrib/etc/docker/daemon.json.in',
  output: 'daemon.json',
  configuration: {
    'BINDIR': bindir,
  },
  install: true,
  install_dir: docker_confdir_path,
)
