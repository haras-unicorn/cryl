{
  self,
  lib,
  config,
  ...
}:

{

  options.cryl = {
    test = {
      modules = lib.mkOption {
        type = lib.types.attrsOf lib.types.raw;
        default = { };
        description = ''
          Cryl test modules.
        '';
      };
    };
  };

  config = {
    systems = self.lib.systems;
    perSystem =
      {
        lib,
        pkgs,
        system,
        ...
      }:
      {
        checks =
          let
            makeTest =
              module:
              let
                makeTest =
                  testModule:
                  pkgs.testers.runNixOSTest {
                    imports = [
                      module
                      testModule
                    ];
                  };
              in
              (makeTest { sshBackdoor.enable = false; })
              // {
                withSshBackdoor = makeTest { sshBackdoor.enable = true; };
              };
          in
          (lib.concatMapAttrs (name: module: {
            "test-${name}" = makeTest {
              imports = [
                module
                self.lib.submodules.test
              ];

              name = "test-${name}";

              cryl.enable = true;
            };
            # TODO: make it work in CI?
            # "test-${name}-sandboxed" = makeTest {
            #   imports = [
            #     module
            #     self.lib.submodules.test
            #   ];

            #   name = "test-${name}-sandboxed";

            #   cryl.enable = true;
            #   cryl.sandboxed = true;
            # };
          }) config.cryl.test.modules)
          // {
            # TODO: make it work in CI?
            # "lint" =
            #   pkgs.runCommand "checks-lint"
            #     {
            #       src = self;
            #       nativeBuildInputs =
            #         (builtins.map (name: self.packages.${system}.${name}) (
            #           builtins.filter (lib.hasPrefix "external") (builtins.attrNames self.packages.${system})
            #         ))
            #         ++ [
            #           self.packages.${system}.flake-root
            #         ];
            #     }
            #     ''
            #       cd "$src"
            #       ${self.lib.scripts.lint}
            #       touch "$out"
            #     '';
          };
      };
  };
}
