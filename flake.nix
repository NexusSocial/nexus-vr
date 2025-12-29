{
  description = "FOSS Social VR development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      nixpkgs-unstable,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        # Create overlay for unstable packages
        unstableOverlay = final: prev: {
          unstable = import nixpkgs-unstable {
            inherit system;
            config.allowUnfree = true;
          };
        };

        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
          };
          overlays = [ unstableOverlay ];
        };

        # Get the fenix packages for this system
        fenixPkgs = fenix.packages.${system};

        # Create a Rust toolchain from rust-toolchain.toml
        selectedRustToolchain = fenixPkgs.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-sqSWJDUxc+zaz1nBWMAJKTAGBuGWP25GCftIOlCEAtA=";
        };

        # Libraries needed for Bevy and VR development
        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          python3
        ];

        buildInputs = with pkgs; [
          # Audio libraries
          alsa-lib

          # Graphics and windowing
          vulkan-loader
          vulkan-headers
          vulkan-tools
          libxkbcommon
          wayland

          # X11 libraries
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr

          # OpenXR support
          openxr-loader

          # Additional libraries that Bevy might need
          udev
          fontconfig
          freetype

          # Networking
          openssl

          # Development tools
          git

          # Trunk for web builds (if needed)
          trunk

          # WASM target for web builds
          wasm-pack
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          packages = with pkgs; [
            selectedRustToolchain
            # Add any additional development tools here
            cargo-deny
            cargo-edit
            cargo-outdated
            cargo-watch
            rust-analyzer
          ];

          # Environment variables for proper library linking
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          # Vulkan setup
          VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";

          shellHook = '''';
        };
      }
    );
}
