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
          ];
        
        shellHook = ''
          exec zsh
          '';
        RUST_LOG = "spin=trace";
        WASMTIME_BACKTRACE_DETAILS = "1";
       CPATH = "/nix/store/4q9wv7pgvnmxlhaknp2lns2d0gmznkdq-tinygo-0.30.0/share/tinygo/lib/wasi-libc/sysroot/include";
        };
      }
    );
}
