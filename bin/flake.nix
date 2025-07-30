{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    systems.url = "systems";
  };

  outputs = {
    nixpkgs,
    crane,
    systems,
    fenix,
    ...
  }: let
    eachSystem = f: nixpkgs.lib.genAttrs (import systems) (system: f nixpkgs.legacyPackages.${system});
  in {
    packages = eachSystem (pkgs: let
      craneLib = crane.mkLib pkgs;
    in {
      default = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
      };
    });

    devShells = eachSystem ({
      pkgs,
      system,
      ...
    }: {
      default = pkgs.mkShell {
        buildInputs = let
          f = fenix.packages.${system};
        in [
          (f.complete.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ])
        ];
      };
    });
  };
}
