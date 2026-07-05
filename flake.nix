{
  description = "racho - A Rust Kernel for RISC-V 64";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fenixPkgs = fenix.packages.${system};

        toolchain =
          with fenixPkgs;
          combine [
            minimal.cargo
            minimal.rustc
            targets.riscv64gc-unknown-none-elf.latest.rust-std
            latest.rust-src
            latest.llvm-tools-preview
            latest.rustfmt-preview
            latest.clippy-preview
          ];
      in
      {
        devShells.default = pkgs.mkShell {
          name = "racho-devshell";

          nativeBuildInputs = [
            toolchain
            pkgs.qemu
            pkgs.gdb-multiarch
            pkgs.python3
            pkgs.gnumake
          ];

          shellHook = ''
            TOOLS_DIR="$PWD/.nix-tools"
            export PATH="$TOOLS_DIR/bin:$PATH"

            if [ ! -f "$TOOLS_DIR/bin/rust-objcopy" ]; then
              echo ">> Installing cargo-binutils (one-time setup)..."
              mkdir -p "$TOOLS_DIR"
              cargo install cargo-binutils --root "$TOOLS_DIR" --quiet
            fi

            echo ""
            echo "== racho - RISC-V 64 Kernel =="
            echo ""
            rustc --version
            cargo --version
            echo ""
            echo "  Build:  make -C os kernel"
            echo "  Run:    make -C os run"
            echo ""
          '';
        };
      }
    );
}
