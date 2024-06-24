{
  description = "A simple rust flake using rust-overlay and craneLib";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    crane,
    flake-utils,
    nixpkgs,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        inherit (pkgs) lib;

        stableToolchain = pkgs.rust-bin.stable.latest.default;
        stableToolchainWithRustAnalyzer = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "rust-analyzer"];
          # Extra targets if required
          # targets = [
          #   "x86_64-unknown-linux-gnu"
          #   "x86_64-unknown-linux-musl"
          #   "x86_64-apple-darwin"
          #   "aarch64-apple-darwin"
          # ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain stableToolchain;
        craneLibLLvmTools = craneLib.overrideToolchain (pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "cargo"
            "llvm-tools"
            "rustc"
          ];
        });

        src = craneLib.path ./.;
        commonArgs = {
          inherit src;
          buildInputs = with pkgs;
            []
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              libiconv
              # pkgs.darwin.apple_sdk.frameworks.Security
              # pkgs.darwin.apple_sdk.frameworks.CoreServices
              # pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ]; # Inputs required for the TARGET system
          # nativeBuildInputs = []; # Intputs required for the HOST system
          # This is often requird for any ffi based packages that use bindgen
          # LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          # For using pkg-config that many libraries require
          # PKG_CONFIG_PATH = lib.makeSearchPath "lib/pkgconfig" (with pkgs;[ openssl.dev zlib.dev ]);
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in {
        checks =
          {
            ansi-to-tui-clippy = craneLib.cargoClippy (commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              });
            ansi-to-tui-fmt = craneLib.cargoFmt {
              inherit src;
            };
            ansi-to-tui-nextest = craneLib.cargoNextest (commonArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
              });
          }
          // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            ansi-to-tui-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs
              // {
                inherit cargoArtifacts;
              });
          };

        devShells.default =
          (craneLib.overrideToolchain stableToolchainWithRustAnalyzer).devShell
          ({
              buildInputs = [];
              nativeBuildInputs = [];
              packages = with pkgs; [
                cargo-nextest
                cargo-criterion
              ];
            }
            // commonArgs);
      }
    );
}
