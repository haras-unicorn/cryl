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
      devShells.default = pkgs.mkShell {
        packages = [
          self.packages.${system}.rust
        ]
        ++ (builtins.map (name: self.packages.${system}.${name}) (
          builtins.filter (lib.hasPrefix "external") (builtins.attrNames self.packages.${system})
        ))
        ++ (builtins.map (name: self.packages.${system}.${name}) (
          builtins.filter (lib.hasPrefix "script") (builtins.attrNames self.packages.${system})
        ))
        ++ (builtins.map (name: self.packages.${system}.${name}) (
          builtins.filter (lib.hasPrefix "build-input") (builtins.attrNames self.packages.${system})
        ));
      };
    };
}
