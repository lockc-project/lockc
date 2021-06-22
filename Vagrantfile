Vagrant.configure("2") do |config|
  config.vm.box = "opensuse/Tumbleweed.x86_64"

  config.vm.synced_folder ".", "/vagrant", disabled: true
  config.vm.synced_folder ".", "/home/vagrant/enclave", type: "rsync",
                          owner: "vagrant", group: "users",
                          rsync__exclude: [".git/", "out/", "target/"]

  config.vm.define "control-plane", primary: true do |cp|
    cp.vm.hostname = "control-plane.local"
    cp.vm.provider :libvirt do |libvirt|
      libvirt.cpus = 4
      libvirt.memory = 4096
    end
    cp.vm.provision "shell", path: "contrib/vagrant/vagrant-fix.sh"
    cp.vm.provision "shell", path: "contrib/vagrant/base.sh", reboot: true
    cp.vm.provision "shell", path: "contrib/vagrant/build.sh", privileged: false
    cp.vm.provision "shell", path: "contrib/vagrant/control-plane-base.sh", reboot: true
    cp.vm.provision "shell", path: "contrib/vagrant/control-plane.sh"
    cp.vm.provision "shell", path: "contrib/vagrant/kubeconfig.sh", privileged: false
    cp.vm.provision "shell", path: "contrib/vagrant/addons.sh", privileged: false
  end
end
