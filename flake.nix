{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-utils.url = "https://flakehub.com/f/numtide/flake-utils/0.1.*.tar.gz";
    naersk.url = "https://flakehub.com/f/nix-community/naersk/0.1.*.tar.gz";
    cross-naersk.url = "github:icewind1991/cross-naersk";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    rust-overlay = {
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.*.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
    nixvim = {
      url = "github:i-ilak/nixvim-config";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
    };
  };

  outputs =
    { flake-utils
    , naersk
    , cross-naersk
    , nixpkgs
    , treefmt-nix
    , rust-overlay
    , pre-commit-hooks
    , nixvim
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      rustOverlay = final: prev: {
        rustToolchain = prev.rust-bin.stable."1.85.0".default.override {
          extensions = [
            "rust-src"
            "rustfmt"
            "clippy"
            "rust-analyzer"
          ];
        };
      };

      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
          rustOverlay
        ];
      };

      cross-naersk' = pkgs.callPackage cross-naersk { inherit naersk; };

      naerskLib = naersk.lib.${system};

      preCommitCheck = pre-commit-hooks.lib.${system}.run
        {
          src = ./.;
          hooks = {
            elm-format.enable = true;
            nixpkgs-fmt.enable = true;
            clippy = {
              enable = true;
              package = pkgs.rustToolchain;
              settings.allFeatures = true;
            };
          };
          settings = {
            rust = {
              check.cargoDeps = pkgs.rustPlatform.importCargoLock {
                lockFile = ./Cargo.lock;
              };
            };
          };
        };
      treefmtEval.${pkgs.system} = treefmt-nix.lib.evalModule pkgs ./nix/treefmt.nix;
    in
    rec {
      devShells = {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            cargo-deny
            cargo-edit
            cargo-watch
            cargo-audit
            cargo-expand
            cargo-udeps
            cargo-nextest
            cargo-flamegraph
            openssl
            cmake
            protobuf_27
            diesel-cli
            sqlite
            pkg-config
            just
            nixvim.packages.${pkgs.system}.default
            pre-commit-hooks.packages.${system}.default
          ];
          RUSTUP_HOME = "/tmp/rustup";
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          shellHook = ''
            ${preCommitCheck.shellHook}
          '';
        };

        release = pkgs.mkShell {
          nativeBuildInputs = devShells.default.nativeBuildInputs ++ [
            pkgs.cargo-release
          ];
          shellHook = devShells.default.shellHook;
        };
      };

      formatter = treefmtEval.${pkgs.system}.config.build.wrapper;
      checks.pre-commit-check = preCommitCheck;
    }
    );
}
