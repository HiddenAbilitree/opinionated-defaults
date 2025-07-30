{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "systems";
  };

  outputs = {
    systems,
    nixpkgs,
    ...
  }: let
    eachSystem = f: nixpkgs.lib.genAttrs (import systems) (system: f nixpkgs.legacyPackages.${system});
  in {
    devShells = eachSystem (pkgs: let
      mkScript = name: text: let
        script = pkgs.writeShellScriptBin name text;
      in
        script;

      scripts = [
        (mkScript "build" ''bun run build'')
        (mkScript "lint" ''oxlint && eslint_d'')
        (mkScript "lint:fix" ''oxlint --fix --fix-suggestions && eslint_d --fix'')
      ];
    in {
      default = pkgs.mkShell {
        buildInputs = with pkgs;
          [
            bun
            nodejs_24
            oxlint
            eslint_d
          ]
          ++ scripts;
      };
    });
  };
}
