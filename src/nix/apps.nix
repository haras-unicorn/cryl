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
      apps =
        let
          package = self.packages.${system}.default;

          app = {
            type = "app";
            program = lib.getExe package;
            meta.description = "Secret generation tool";
          };
        in
        {
          default = app;
          cryl = app;
        };
    };
}
