{ self, ... }:

# TODO: convert all test to flake tests

let
  crylSelf = self;
in
{
  flake.flakeModules = rec {
    default = cryl;
    cryl =
      {
        self,
        lib,
        config,
        options,
        ...
      }:
      {
        options = {
          flake.cryl = lib.mkOption {
            type = lib.types.submodule (
              { lib, config, ... }:
              {
                imports = [
                  crylSelf.lib.submodules.flakeOrTest
                ];

                nixosConfigurations = builtins.mapAttrs (_: nixos: nixos.config) self.nixosConfigurations;
                defaultSandboxed = true;
              }
            );
            default = { };
            description = "Cryl output for this flake";
          };
        };

        config = lib.mkMerge [
          (lib.mkIf config.cryl.enable {
            flake.nixosModules.cryl =
              {
                lib,
                config,
                options,
                ...
              }:
              let
                cfg = config.cryl;
              in
              {
                options.cryl = lib.mkOption {
                  type = lib.types.submodule {
                    imports = [ crylSelf.lib.submodules.nixosOrHome ];

                    sops.defaultPath = "sops/${config.networking.hostName}.yaml";
                    sops.age.defaultPath = "age/${config.networking.hostName}.txt";
                  };
                  default = { };
                  description = "Cryl config for this nixos configuration";
                };

                config = lib.mkIf cfg.enable (
                  if !(options ? sops) then
                    { }
                  else
                    {
                      sops.defaultSopsFile = "${self}/${cfg.sops.path}";
                    }
                );
              };

            systems = crylSelf.lib.systems;
            perSystem =
              {
                lib,
                pkgs,
                system,
                ...
              }:
              {
                packages = lib.mapAttrs' (
                  name: value:
                  let
                    specification = pkgs.writers.writeTOML "${name}-cryl-specification.toml" value;
                  in
                  {
                    name = "cryl-${name}";
                    value = pkgs.writeShellApplication {
                      name = "cryl-${name}";
                      runtimeInputs = [
                        crylSelf.packages.${system}.cryl
                        crylSelf.packages.${system}.flake-root
                      ];
                      text = ''
                        export out="$(flake-root)"
                        ${config.flake.cryl.shellInvocationForPath} "$@" ${specification}
                      '';
                    };
                  }
                ) config.flake.cyrl.specifications;
              };
          })
          (
            if !(options.flake ? homeModules) then
              { }
            else
              {
                flake.homeModules.cryl =
                  {
                    lib,
                    config,
                    osConfig,
                    options,
                    ...
                  }:
                  let
                    cfg = config.cryl;
                  in
                  {
                    options.cryl = lib.mkOption {
                      type = lib.types.submodule {
                        imports = [ crylSelf.lib.submodules.nixosOrHome ];

                        sops.defaultPath =
                          if osConfig ? cryl && osConfig.cryl.enable then
                            osConfig.cryl.sops.path
                          else
                            "sops/${osConfig.networking.hostName}-${config.home.username}.yaml";

                        sops.age.defaultPath =
                          if osConfig ? cryl && osConfig.cryl.enable then
                            osConfig.cryl.sops.path
                          else
                            "age/${osConfig.networking.hostName}-${config.home.username}.txt";
                      };
                      default = { };
                      description = "Cryl config for this home configuration";
                    };

                    config = lib.mkIf cfg.enable (
                      if !(options ? sops) then
                        { }
                      else
                        {
                          sops.defaultSopsFile = "${self}/${cfg.sops.path}";
                        }
                    );
                  };
              }
          )
        ];
      };
  };
}
