Vagrant.configure("2") do |config|
  config.vm.box = "opensuse/Tumbleweed.x86_64"
  config.vm.box_version = "1.0.20210702"

  config.vm.synced_folder ".", "/vagrant", disabled: true
  config.vm.synced_folder ".", "/home/vagrant/enclave", type: "rsync",
                          owner: "vagrant", group: "users",
                          rsync__exclude: [".git/", "out/", "target/"]

  config.vm.define "control-plane", primary: true do |cp|
    cp.vm.hostname = "control-plane.local"
    cp.vm.provider :libvirt do |libvirt|
      libvirt.cpus = 4
      libvirt.memory = 8192
    end
    cp.vm.provision "shell", path: "contrib/provision/vagrant-fix.sh"
    cp.vm.provision "shell", path: "contrib/provision/base.sh", reboot: true
    cp.vm.provision "shell", path: "contrib/provision/build.sh", privileged: false
    cp.vm.provision "shell", path: "contrib/provision/control-plane-base.sh"
    cp.vm.provision "shell", path: "contrib/provision/control-plane.sh"
    cp.vm.provision "shell", path: "contrib/provision/kubeconfig.sh", privileged: false
    cp.vm.provision "shell", path: "contrib/provision/addons.sh", privileged: false
  end
end
