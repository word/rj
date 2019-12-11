# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  # https://docs.vagrantup.com

  config.vm.box = "punktde/freebsd-120-zfs"
  config.ssh.shell = "/bin/sh"

  # config.vm.network "forwarded_port", guest: 80, host: 8080
  # config.vm.network "forwarded_port", guest: 80, host: 8080, host_ip: "127.0.0.1"
  # config.vm.network "public_network"
  config.vm.network "private_network", ip: "10.33.55.5"


  config.vm.synced_folder ".", "/vagrant", type: "nfs"

  config.vm.provider "virtualbox" do |vb|
    # Display the VirtualBox GUI when booting the machine
    vb.gui = false
    # Customize the amount of memory on the VM:
    vb.memory = "3048"
  end

  config.vm.provision "shell", inline: <<-SHELL
    sysrc rpc_lockd_enable="YES"
    sysrc rpc_statd_enable="YES"
    service lockd start
    service statd start
    pkg update
    pkg install -y rust vim-tiny
  SHELL
end
