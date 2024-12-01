{
  description = "tankrs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        {
          pkgs,
          ...
        }:
        let
          buildInputsArr = with pkgs; [
            udev
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libxcb
            libGL
            vulkan-loader
            vulkan-headers
            alsa-lib
          ];
          nativeBuildInputsArr = with pkgs; [
            rustc
            rust-analyzer
            rustfmt
            clippy
            wayland
            rust-analyzer
            rustfmt
            lldb
            cargo-geiger
            renderdoc
            alsa-lib
            pkg-config
            mold
            clang
            makeWrapper
            lld
            libxkbcommon
            udev
            alsa-lib
            vulkan-loader
            glib
          ];

        in
        {
          devShells.default = pkgs.mkShell rec {
            buildInputs = buildInputsArr;
            nativeBuildInputs = nativeBuildInputsArr;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          };

          packages = {
            default = pkgs.callPackage ./nix/tankrs.nix { inherit inputs buildInputsArr nativeBuildInputsArr; };
          };
        };
    };
}
