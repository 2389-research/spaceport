# ABOUTME: Builds the Spaceport development client and helper for Linux with Nix.
# ABOUTME: Installs launcher metadata and runtime libraries for the packaged app.
{
  description = "Spaceport: a native GPUI client for Herdr";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Build with exactly the toolchain rust-toolchain.toml pins, not whatever
    # rustc the nixpkgs revision happens to carry.
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      inherit (nixpkgs) lib;
      # GPUI's Linux backend is the only one this flake builds; macOS users
      # install a signed Spaceport preview instead.
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems =
        f:
        lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );
      package =
        pkgs:
        let
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
          # Loaded with dlopen at runtime, so the linker never records them.
          runtimeLibraries = [
            pkgs.vulkan-loader
            pkgs.wayland
          ];
        in
        rustPlatform.buildRustPackage {
          pname = "spaceport";
          version = "0-unstable-${lib.substring 0 8 (self.lastModifiedDate or "19700101")}";
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./rust-toolchain.toml
              ./crates
              ./assets
              # The agent skill is compiled into the executable.
              ./skills
              ./scripts/release/org.sheprd.Sheprd.desktop
            ];
          };
          cargoLock.lockFile = ./Cargo.lock;
          # The app starts chats with sheprd-acp beside its executable.
          cargoBuildFlags = [
            "-p"
            "sheprd-gpui"
            "-p"
            "sheprd-acp"
          ];
          # The optimized CLI suite CI's main-branch build runs; the rest of
          # the workspace suite needs a desktop or a live daemon.
          cargoTestFlags = [
            "-p"
            "sheprd-gpui"
            "--test"
            "cli"
          ];
          nativeBuildInputs = [
            pkgs.desktop-file-utils
            pkgs.makeWrapper
            pkgs.pkg-config
            rustPlatform.bindgenHook
          ];
          buildInputs = [
            pkgs.alsa-lib
            pkgs.fontconfig
            pkgs.freetype
            pkgs.libxkbcommon
            pkgs.libxcb
          ]
          ++ runtimeLibraries;
          # Nix owns the store path and upgrades come from the flake.
          postInstall = ''
            install -Dm644 scripts/release/org.sheprd.Sheprd.desktop \
              $out/share/applications/spaceport.development.desktop
            install -Dm644 assets/icons/sheprd-linux.svg \
              $out/share/icons/hicolor/scalable/apps/spaceport.svg
            desktop-file-validate $out/share/applications/spaceport.development.desktop
          '';
          postFixup = ''
            patchelf --add-rpath ${lib.makeLibraryPath runtimeLibraries} $out/bin/spaceport
            wrapProgram $out/bin/spaceport \
              --prefix PATH : ${lib.makeBinPath [ pkgs.wl-clipboard pkgs.xclip ]}
          '';
          meta = {
            description = "Native GPUI client for Herdr";
            homepage = "https://github.com/2389-research/spaceport";
            license = lib.licenses.asl20;
            mainProgram = "spaceport";
            platforms = systems;
          };
        };
    in
    {
      packages = forAllSystems (pkgs: {
        default = package pkgs;
        spaceport = package pkgs;
      });
      overlays.default = final: _prev: {
        spaceport = self.packages.${final.stdenv.hostPlatform.system}.default;
      };
    };
}
