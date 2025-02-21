Vagrant.configure("2") do |config|
  config.vm.box = "perk/ubuntu-2204-arm64"

  config.vm.synced_folder ".", "/vagrant",
    type: "rsync",
    rsync__exclude: [".git/", "target/", "*.swp"],
    rsync__auto: true

  config.vm.provision "shell", inline: <<-SHELL
    sudo locale-gen en_US.UTF-8

    export LANG=en_US.UTF-8
    export LC_ALL=en_US.UTF-8

    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libcgroup-dev

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  SHELL
end
