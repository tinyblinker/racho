{
  description = "racho — RISC-V 64 Rust kernel dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      fhs = pkgs.buildFHSEnv {
        name = "racho-fhs";
        targetPkgs =
          pkgs: with pkgs; [
            rustup # rustup needs FHS to install packages
            qemu
            cargo-binutils
            python3
            git
            gdb
            gcc
            zlib
            openssl
            pkg-config
            cacert
          ];
        runScript = "bash";
        profile = ''
          export RACHO_FHS_GUARD=1
          echo "==> racho dev shell (FHS)"
          echo "    Rust:  rust-toolchain.toml (auto-installs on first cargo run)"
          echo "    QEMU:  qemu-system-riscv64"
          echo "    GDB:   gdb"
          echo "    Build: cd os && make run"
          echo ""
        '';
      };
    in
    {
      devShells.${system}.default = fhs.env;
      packages.${system}.env = fhs;
    };
}
