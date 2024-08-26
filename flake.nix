{
  description = "spin-demo";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: 
    flake-utils.lib.eachDefaultSystem (system: 
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
      in {
        devShells.default = pkgs.mkShell {
          name = "spin-spinKube";

          packages = [
            toolchain
            pkgs.tinygo
            pkgs.nodePackages.webpack-cli
            pkgs.python310
            pkgs.python310Packages.pip
            pkgs.python310Packages.setuptools
            pkgs.sops
            pkgs.wash-cli
          ];
        
        shellHook = ''
          export PIP_PREFIX=$(pwd)/_build/pip_packages
          export PYTHONPATH="$PIP_PREFIX/${pkgs.python310.sitePackages}:$PYTHONPATH"
          export PATH="$PIP_PREFIX/bin:$PATH"
          unset SOURCE_DATE_EPOCH
          exec zsh
          '';

        RUST_LOG = "spin=trace";
        WASMTIME_BACKTRACE_DETAILS = "1";
        CPATH="${pkgs.tinygo}/share/tinygo/lib/wasi-libc/sysroot/include";
        };
      }
    );
}
