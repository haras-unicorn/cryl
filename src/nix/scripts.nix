{ self, ... }:

{
  systems = self.lib.systems;
  perSystem =
    {
      lib,
      pkgs,
      system,
      ...
    }:
    {
      packages = lib.mapAttrs' (name: value: {
        name = "script-${name}";
        value = pkgs.writeShellApplication {
          name = "dev-${name}";
          runtimeInputs =
            (builtins.map (name: self.packages.${system}.${name}) (
              builtins.filter (lib.hasPrefix "external") (builtins.attrNames self.packages.${system})
            ))
            ++ [
              self.packages.${system}.flake-root
            ];
          text = value;
        };
      }) self.lib.scripts;
    };
}
