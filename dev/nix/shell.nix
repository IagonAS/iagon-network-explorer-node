{
  self,
  inputs,
  ...
}: {
  perSystem = {
    inputs',
    self',
    pkgs,
    system,
    ...
  }: let
    isDarwin = pkgs.lib.hasSuffix "darwin" system;
    fenixPkgs = inputs'.fenix.packages;
    rustToolchain = with fenixPkgs;
      fromToolchainFile {
        file = ../../rust-toolchain.toml;
        # Probably should be a flake input instead
        sha256 = "VZZnlyP69+Y3crrLHQyJirqlHrTtGTsyiSnZB8jEvVo=";
      };
    packages = with pkgs;
      [
        coreutils
        pkg-config
        protobuf
        rustToolchain
        nodejs
        nodePackages.npm
        pkgs.libiconv
        pkgs.openssl
        gnumake
        gawk
        cargo-edit
        perl
      ]
      ++ (
        if isDarwin
        then
           [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ]
        else [ pkgs.clang ]
      );
    env = [
      {
        name = "RUST_SRC_PATH";
        value = "${rustToolchain}/lib/rustlib/src/rust/library";
      }
      { name = "LIBCLANG_PATH"; value = "${pkgs.libclang.lib}/lib"; }
      { name = "LD_LIBRARY_PATH"; value = "${rustToolchain}/lib"; }
      { name = "ROCKSDB_LIB_DIR"; value = "${pkgs.rocksdb}/lib/"; }
      { name = "OPENSSL_NO_VENDOR"; value = 1; }
      { name = "OPENSSL_DIR"; value = "${pkgs.openssl.dev}"; }
      { name = "OPENSSL_INCLUDE_DIR"; value = "${pkgs.openssl.dev}/include"; }
      { name = "OPENSSL_LIB_DIR"; value = "${pkgs.openssl.out}/lib"; }

    ];
    # Main Categories which can include pkgs, or devShell-like sets
    # for commands and helpers
    commands = self.lib.categorize [
      {
        help = "Earthly, an easy to use CI tool";
        name = "earthly";
        package = pkgs.earthly;
        category = "CI/CD";
        pkgs = [
          pkgs.earthly
          pkgs.awscli2
          pkgs.kubectl
          pkgs.kubernetes-helm
        ];
      }
      {
        category = "Rust";
        pkgs = [
          {
            name = "check";
            help = "Check rustc and clippy warnings";
            command = ''
              set -x
              cargo check --all-targets
              cargo clippy --all-targets
            '';
          }
          {
            help = "Automatically fix rustc and clippy warnings";
            name = "fix";
            command = ''
              set -x
              cargo fix --all-targets --allow-dirty --allow-staged
              cargo clippy --all-targets --fix --allow-dirty --allow-staged
            '';
          }
        ];
      }
      {
        category = "Partner Chains";
        pkgs = [
          {
            name = "cardano-cli";
            help = "CLI v10.1.2 that is used in partner-chains dependency stack";
            # This command has some eval because of IFD
            command = "${self'.packages.cardano-cli}/bin/cardano-cli latest $@";
          }
        ];
      }
    ];
    extraCommands =
      commands
      ++ self.lib.categorize [
        {
          category = "Partner Chains";
          pkgs = [
            {
              name = "partnerchains-stack";
              help = "Run a containerless stack of all of the dependencies. Use -n <network> to specify networks";
              command = ''
                ${self'.packages.partnerchains-stack}/bin/partnerchains-stack $@
              '';
            }
            {
              name = "pc-contracts-cli";
              help = "CLI to interact with Partner Chains Smart Contracts";
              command = ''
                ${self'.packages.pc-contracts-cli}/dist/pc-contracts-cli $@
              '';
            }
          ];
        }
      ];
  in {
    devshells.default = {
      inherit packages env commands;
      name = "Partner Chains Substrate Node Devshell";
    };
    devshells.process-compose = {
      inherit packages env;
      commands = extraCommands;
      name = "Partner Chains Substrate Node Devshell with whole stack";
    };
    devshells.smart-contracts = {
      inherit packages env;
      commands = commands ++ [
        {
          category = "Partner Chains";
          name = "pc-contracts-cli";
          help = "CLI to interact with Partner Chains Smart Contracts";
          command = ''
            ${self'.packages.pc-contracts-cli}/dist/pc-contracts-cli $@
          '';
        }
      ];
      name = "Partner Chains Substrate Node Devshell with Smart Contracts CLI";
    };
  };
}
