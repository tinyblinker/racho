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
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustup
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
        shellHook = ''
          echo "==> racho dev shell"
          echo "    Rust:  rust-toolchain.toml (auto-installs on first cargo run)"
          echo "    QEMU:  qemu-system-riscv64"
          echo "    GDB:   gdb"
          echo "    Build:  cargo build -p user_lib --release && cargo run -p kernel --release"
        echo "    Debug:  RACHO_GDB=1 cargo run -p kernel --release"
        echo "    GDB:    ./tools/gdb_client.sh"
          echo ""
        '';
      };
    };
}
