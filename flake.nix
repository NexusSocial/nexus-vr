{
  description = "nexus flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # eachDefaultSystem and other utility functions
    utils.url = "github:numtide/flake-utils";
    # Replacement for rustup
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, fenix, }:
    # This helper function abstracts over the host platform.
    # See https://github.com/numtide/flake-utils#eachdefaultsystem--system---attrs
    utils.lib.eachDefaultSystem (system:
      let
        makeP = system: import nixpkgs {
          system = system;
          config = {
            android_sdk.accept_license = true;
            allowUnfree = true;
          };
        };
        p = {
          native = makeP "${system}"; # The host environment
          android = makeP "aarch64-android";
          arm-linux = makeP "aarch64-linux";
          x86-linux = makeP "x86-linux";
          arm-macos = makeP "arm-macos";
          x86-macos = makeP "x86-macos";
        };
        # Brings in the rust toolchain from the standard file
        # that rustup/cargo uses.
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-z8mcLro/fWE+WzsB5fiirL0Ov9/tMXnIdmgbZvZc2aA=";
        };
        rustPlatform = p.native.makeRustPlatform {
          inherit (rustToolchain) cargo rustc;
        };
      in
      # See https://nixos.wiki/wiki/Flakes#Output_schema
      {
        # `nix develop` pulls all of this in to become your shell.
        devShells.default = p.native.mkShell {
          buildInputs = (with p.native; [
            # necessary build tools
            zig
            cargo-zigbuild
            cmake # for openxr
            # unwrapped to avoid nix messing with our provided values for PKG_CONFIG
            pkg-config-unwrapped

            # dev tools
            cargo-binutils
            cargo-deny
            cargo-expand
            nixpkgs-fmt
          ]) ++ [
            rustPlatform.bindgenHook
            rustToolchain
          ] ++ p.native.lib.optionals (p.native.stdenv.isDarwin) (with p.native.darwin.apple_sdk.frameworks; [
            # # This is missing on mac m1 nix, for some reason.
            # # see https://stackoverflow.com/a/69732679
            p.native.libiconv
            Cocoa
          ]);
        };
        # This only formats the nix files.
        formatter = p.native.nixpkgs-fmt;
      }
    );
}
