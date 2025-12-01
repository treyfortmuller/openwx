{
  description = "OpenWeather API wrapper library, plus a small test CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
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
      flake-utils,
    }:

    flake-utils.lib.eachSystem [ "aarch64-linux" "x86_64-linux" ] (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust = pkgs.rust-bin.stable.latest.default.override {
          # Optional extensions can be added here
          extensions = [ ]; # e.g. "llvm-tools-preview"
          targets = [ ]; # e.g. "thumbv7em-none-eabihf"
        };

        # Create a rustPlatform using oxalica's toolchain
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust;
          rustc = rust;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust
            pkg-config
            openssl
          ];

          # Optional: helpful environment variables for Rust dev
          # RUST_BACKTRACE = "1";
          RUST_SRC_PATH = rustPlatform.rustLibSrc;

          shellHook = ''
            echo "🦀 Evolved into a crab again... shit."
            rustc --version
          '';
        };

        packages.default =
          let
            cargoToml = builtins.fromTOML (builtins.readFile "${self}/Cargo.toml");
          in
          rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            src = self;

            cargoLock = {
              lockFile = "${self}/Cargo.lock";

              # Nix needs inputs to be content-addressable and git dependencies are not,
              # even for fixed revs in your Cargo.toml so we need to specify these.
              outputHashes = { };
            };

            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ];
          };

        formatter = pkgs.nixfmt-tree;
      }
    );
}
