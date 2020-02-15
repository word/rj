# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  # https://docs.vagrantup.com

  config.vm.box = "punktde/freebsd-121-zfs"

  config.ssh.shell = "/bin/sh"

  # config.vm.network "forwarded_port", guest: 80, host: 8080
  # config.vm.network "forwarded_port", guest: 80, host: 8080, host_ip: "127.0.0.1"
  # config.vm.network "public_network"
  config.vm.network "private_network", ip: "10.33.55.5"


  # config.vm.synced_folder ".", "/vagrant", type: "nfs"
  config.vm.synced_folder ".", "/vagrant", type: "rsync",
      rsync__exclude: [ "target/", ".vagrant/", ".git/" ],
      rsync__args: ["-a", "--delete"]

  # config.vm.provider :libvirt do |libvirt|
  #   libvirt.cpus = 2
  #   libvirt.memory = "3048"
  # end

  config.vm.provider "virtualbox" do |vb|
    # Display the VirtualBox GUI when booting the machine
    vb.gui = false
    # Customize the amount of memory on the VM:
    vb.memory = "4048"
    # CPUs
    vb.cpus = 2
    # Disable audio
    vb.customize ["modifyvm", :id, "--audio", "none"]
    # Disable USB
    vb.customize ["modifyvm", :id, "--usb", "off"]
  end

  config.vm.provision "shell", inline: <<-SHELL
    #sysrc rpc_lockd_enable="YES"
    #sysrc rpc_statd_enable="YES"
    #service lockd start
    #service statd start
    pkg update
    pkg install -y rust vim-tiny

    # jail networking
    echo 'nat on em0 from 10.11.11.0/24 to any -> (em0)' > /etc/pf.conf
    echo 'pass all' >> /etc/pf.conf
    sysrc pf_enable=yes
    service pf start
    sysrc gateway_enable=yes
    sysctl net.inet.ip.forwarding=1
  SHELL
end
