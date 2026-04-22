{
  description = "Two Rust Logos modules demonstrating inter-module IPC: rust_provider_module (callee) and rust_caller_module (caller via logos-rust-sdk)";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder/c_ffi";
    logos-module-builder.inputs.logos-cpp-sdk.url = "github:logos-co/logos-cpp-sdk/c_ffi";
    logos-module-client.url = "github:logos-co/logos-module-client/new_api_test";
    logos-rust-sdk.url = "github:logos-co/logos-rust-sdk/new_api_test";
    logos-logoscore-cli.url = "github:logos-co/logos-logoscore-cli";
    nixpkgs.follows = "logos-module-builder/nixpkgs";
  };

  outputs = inputs@{ self, logos-module-builder, logos-module-client, logos-rust-sdk, logos-logoscore-cli, nixpkgs, ... }:
    let
      mkModule = logos-module-builder.lib.mkLogosModule;
      systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ];
      forAllSystems = fn: nixpkgs.lib.genAttrs systems fn;

      # ── Module A: rust_provider_module ───────────────────────────────────────
      provider = mkModule {
        src = ./rust-provider-module;
        configFile = ./rust-provider-module/metadata.json;
        flakeInputs = inputs;
        preConfigure = ''
          echo "=== Building Rust provider library ==="
          export HOME=$TMPDIR
          export CARGO_HOME=$TMPDIR/cargo
          mkdir -p $CARGO_HOME

          pushd rust-lib
          cargo build --release --offline
          popd

          mkdir -p lib
          cp rust-lib/target/release/librust_provider.a lib/
          cp rust-lib/include/rust_provider.h lib/
          echo "=== Rust provider library built and staged ==="
        '';
      };

    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          moduleClient    = logos-module-client.packages.${system}.logos-module-client;
          moduleClientLib = logos-module-client.packages.${system}.logos-module-client-lib;

          # ── Rust caller staticlib ───────────────────────────────────────────
          # Assemble a source tree matching the Cargo.toml path layout:
          #   rust-lib/          (Cargo.toml, src/, include/)
          #   logos-rust-sdk-src/ (the SDK crate)
          callerRustSrc = pkgs.runCommand "rust-caller-src" {} ''
            mkdir -p $out
            cp -r ${./rust-caller-module/rust-lib} $out/rust-lib
            cp -r ${logos-rust-sdk} $out/logos-rust-sdk-src
          '';

          callerRustLib = pkgs.rustPlatform.buildRustPackage {
            pname = "rust_caller";
            version = "1.0.0";
            src = callerRustSrc;
            sourceRoot = "rust-caller-src/rust-lib";
            cargoHash = "sha256-3JjEOv7ecypK+po8JprqwlrbGWl+LpbhaGii2lMj8Vs=";
            doCheck = false;
          };

          # ── Module B: rust_caller_module ──────────────────────────────────────
          caller = mkModule {
            src = ./rust-caller-module;
            configFile = ./rust-caller-module/metadata.json;
            flakeInputs = { rust_provider_module = provider; } // inputs;

            extraBuildInputs = [ moduleClientLib ];

            preConfigure = ''
              echo "=== Staging pre-built Rust caller library ==="
              mkdir -p lib
              cp ${callerRustLib}/lib/librust_caller.a lib/
              cp rust-lib/include/rust_caller.h lib/
              echo "=== Rust caller library staged ==="

              export LOGOS_MODULE_CLIENT_ROOT="${moduleClient}"
            '';
          };

        in
        let
          providerInstall = provider.packages.${system}.install;
          callerInstall   = caller.packages.${system}.install;

          modulesDir = pkgs.runCommand "rust-example-modules-dir" {} ''
            mkdir -p $out
            for src in ${providerInstall} ${callerInstall}; do
              cp -rL "$src"/modules/* $out/ 2>/dev/null || true
            done
          '';
        in
        {
          rust_provider_module = provider.packages.${system}.default;
          rust_caller_module   = caller.packages.${system}.default;

          rust_provider_module_install = providerInstall;
          rust_caller_module_install   = callerInstall;
          modules = modulesDir;

          default = pkgs.symlinkJoin {
            name = "logos-rust-example-modules";
            paths = [
              provider.packages.${system}.default
              caller.packages.${system}.default
            ];
          };
        }
      );

      checks = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          logoscore = logos-logoscore-cli.packages.${system}.default;
          modulesDir = self.packages.${system}.modules;
        in
        {
          ipc-test = pkgs.runCommand "rust-modules-ipc-test" {
            nativeBuildInputs = [ logoscore ]
              ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.qt6.qtbase ];
          } ''
            mkdir -p $out
            export QT_QPA_PLATFORM=offscreen
            echo "=== Testing Rust module IPC ==="
            logoscore --quit-on-finish \
              -m ${modulesDir} \
              -l rust_caller_module \
              -c "rust_caller_module.call_add(5, 3)" \
              -c "rust_caller_module.call_multiply(4, 6)" \
              -c "rust_caller_module.call_greet(World)"
            echo "IPC test passed" > $out/result.txt
          '';
        }
      );
    };
}
