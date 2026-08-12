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

  outputs =
    {
      nixpkgs,
      crane,
      systems,
      fenix,
      ...
    }:
    let
      eachSystem =
        f:
        nixpkgs.lib.genAttrs (import systems) (
          system:
          f {
            inherit system;
            pkgs = nixpkgs.legacyPackages.${system};
          }
        );
    in
    {
      packages = eachSystem (
        { pkgs, ... }:
        let
          craneLib = crane.mkLib pkgs;
        in
        {
          default = craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./bin;
          };
        }
      );

      devShells = eachSystem (
        { pkgs, system }:
        let
          mkScript =
            name: text:
            let
              script = pkgs.writeShellScriptBin name text;
            in
            script;

          scripts = [
            (mkScript "build" "bun run build")
            (mkScript "demo" ''nix-shell -p vhs difftastic --run "vhs demo.tape --output ./assets/demo.gif"'')
          ];

          rust = fenix.packages.${system}.latest.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ];
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              rust
              pkgs.bun
              pkgs.nodejs_26
            ]
            ++ scripts;
          };
        }
      );
    };
}
