CPUS = (ENV['LOCKC_VAGRANT_CPUS'] || 4).to_i
MEMORY = (ENV['LOCKC_VAGRANT_MEMORY'] || 8192).to_i

Vagrant.configure("2") do |config|
  config.vm.box = "generic/ubuntu2110"

  config.vm.synced_folder ".", "/vagrant", type: "rsync",
    rsync__exclude: "target/"

  config.vm.provider "virtualbox" do |v|
    v.cpus = CPUS
    v.memory = MEMORY
    v.customize ["modifyvm", :id, "--audio", "none"]
  end
  config.vm.provider "libvirt" do |libvirt|
    libvirt.cpus = CPUS
    libvirt.memory = MEMORY
  end

  config.vm.provision "shell", inline: <<-SHELL
    wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
    add-apt-repository 'deb http://apt.llvm.org/impish/ llvm-toolchain-impish-14 main'
    apt-get update
    apt-get install -y \
        build-essential \
        clang-14 \
        lldb-14 \
        lld-14 \
        llvm-14 \
        llvm-14-dev
    sed -i 's/GRUB_CMDLINE_LINUX=\"\"/GRUB_CMDLINE_LINUX=\"lsm=lockdown,yama,bpf\"/' /etc/default/grub
    update-grub
  SHELL
  config.vm.provision :reload
  config.vm.provision "shell", privileged: false, inline: <<-SHELL
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    rustup toolchain install nightly --component rust-src
    cargo install bpf-linker

    pushd /vagrant
    cargo xtask build-ebpf
    cargo build
    cargo xtask install
    popd
  SHELL
  config.vm.provision "shell", inline: <<-SHELL
    systemctl enable --now lockc
  SHELL
  config.vm.define "server" do |server|
    server.vm.network "private_network", ip: "192.168.33.10"
    server.vm.provision "shell", inline: <<-SHELL
      curl -sfL https://get.k3s.io | K3S_TOKEN=mynodetoken sh -
    SHELL
  end

  # TODO(vadorovsky): Enble agent when we deploy lockc with helm.
  # config.vm.define "agent" do |agent|
  #   agent.vm.network "private_network", ip: "192.168.33.11"
  #   agent.vm.provision "shell", inline: <<-SHELL
  #     curl -sfL https://get.k3s.io | K3S_URL=https://192.168.33.10:6443 K3S_TOKEN=mynodetoken sh -
  #   SHELL
  # end
end