{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    squads-multisig-client.url = "github:Denommus/squads-multisig-client";
    squads-multisig-client.inputs.nixpkgs.follows = "nixpkgs";
    squads-multisig-client.inputs.flake-utils.follows = "flake-utils";
    squads-multisig-client.inputs.naersk.follows = "naersk";
    squads-multisig-client.inputs.rust-overlay.follows = "rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      naersk,
      squads-multisig-client,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };

        squads-client = pkgs.callPackage ./squads-client.nix { inherit naersk; };

        shell = pkgs.mkShell {
          inputsFrom = [ squads-client ];

          packages = [
            squads-multisig-client.packages.${system}.default
            pkgs.cargo-machete
          ];
        };
      in
      {
        packages = {
          inherit squads-client;
          default = squads-client;
        };

        devShells.default = shell;
      }
    );
}
