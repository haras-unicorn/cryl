{ self, ... }:

{
  systems = self.lib.systems;
  perSystem =
    { lib, pkgs, ... }:
    {
      packages =
        let
          docs =
            pkgs.runCommand "cryl-docs"
              {
                src = self;
                nativeBuildInputs = [ pkgs.mdbook ];
              }
              ''
                mdbook build -d "$out" "$src/docs"
              '';

          makeModuleOptionsMarkdownPackages =
            name: module:
            let
              packages = pkgs.nixosOptionsDoc {
                transformOptions =
                  opt:
                  opt
                  // {
                    visible = opt.visible or true && (builtins.head opt.loc) != "_module";
                    declarations = [ ];
                  };
                options =
                  let
                    eval = lib.evalModules {
                      modules = [
                        lib.types.noCheckForDocsModule
                        module
                      ];
                    };
                  in
                  eval.options;
              };
            in
            {
              ${name} = packages.optionsCommonMark;
            };
        in
        (makeModuleOptionsMarkdownPackages "docs-flake-or-test-submodule" self.lib.submodules.flakeOrTest)
        // (makeModuleOptionsMarkdownPackages "docs-nixos-or-home-submodule" self.lib.submodules.nixosOrHome)
        // {
          inherit docs;
        };
    };
}
