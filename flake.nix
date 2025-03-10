{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # Rust toolchains in nix
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Nix helpers
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat = {
      url = "github:input-output-hk/flake-compat/fixes";
      flake = false;
    };
    blank.url = "github:input-output-hk/empty-flake";
    process-compose.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";

    # Partner Chains deps
    smart-contracts = {
      url = "github:input-output-hk/partner-chains-smart-contracts/v7.0.1";
      flake = false;
    };
    cardano-node = {
      url = "github:IntersectMBO/cardano-node/10.1.2";
      flake = false;
    };
    cardano-dbsync = {
      url = "github:IntersectMBO/cardano-db-sync/13.5.0.2";
      flake = false;
    };
    kupo = {
      url = "github:CardanoSolutions/kupo/v2.9.0";
      flake = false;
    };
    ogmios = {
      url = "github:CardanoSolutions/ogmios/v6.9.0";
      flake = false;
    };
    configurations = {
      url = "github:input-output-hk/cardano-configurations";
      flake = false;
    };
  };
  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-darwin"];
      imports = [
        inputs.devshell.flakeModule
        inputs.process-compose.flakeModule
        ./dev/nix/shell.nix
        ./dev/nix/packages
        ./dev/nix/processes.nix
      ];
      flake.lib = import ./dev/nix/lib.nix {inherit (nixpkgs) lib;};
    };
  nixConfig = {
    allow-import-from-derivation = true;
    accept-flake-config = true;
    extra-substituters = [
      "https://nix-community.cachix.org"
      "https://cache.iog.io"
      "https://cache.sc.iog.io"
    ];
    extra-trusted-public-keys = [
      "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "cache.sc.iog.io:b4YIcBabCEVKrLQgGW8Fylz4W8IvvfzRc+hy0idqrWU="
    ];
  };
}
