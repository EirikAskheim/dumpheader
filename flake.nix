{
  description = "A devShell example";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustVersion = pkgs.rust-bin.beta.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
        myRustBuild = rustPlatform.buildRustPackage {
          pname = "dumpheader";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };
        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = myRustBuild.pname;
          tag = "latest";
          config = { Cmd = [ "${myRustBuild}/bin/dumpheader" ]; };
        };
      in
      with pkgs;
      {
        packages = {
         rustPackage = myRustBuild;
         docker = dockerImage;
        };
        defaultPackage = dockerImage;
        devShells.default = mkShell {
          buildInputs = [
            openssl
            pkg-config
            eza
            fd
            rust-bin.beta.latest.default
          ];
          shellHook = ''
            alias ls=eza
            alias find=fd
          '';
        };
      }
    );
}

