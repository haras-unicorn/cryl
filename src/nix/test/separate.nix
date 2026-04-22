{
  cryl.test.modules.separate =
    { lib, ... }:
    let
      homeText = "Hello home from cryl!";
      homeTextShellArg = lib.escapeShellArg homeText;
      nixosText = "Hello nixos from cryl!";
      nixosTextShellArg = lib.escapeShellArg nixosText;
      name = "cryl-test-text";
      home = "/home/test";
      homePath = "${home}/${name}";
      homePathShellArg = lib.escapeShellArg homePath;
      nixosPath = "/etc/test/${name}";
      nixosPathShellArg = lib.escapeShellArg nixosPath;
      pass = "test";
      passShellArg = lib.escapeShellArg pass;
    in
    {
      nodes.machine =
        { pkgs, ... }:
        {
          users.groups.test = { };

          users.users.test = {
            inherit home;
            group = "test";
            isNormalUser = true;
            initialPassword = pass;
          };

          services.openssh.enable = true;
          environment.systemPackages = [ pkgs.sshpass ];

          sops.secrets.${name} = {
            path = nixosPath;
          };

          cryl.enable = true;

          cryl.sops.path = "sops/nixos.yaml";
          cryl.sops.age.path = "age/nixos.yaml";

          cryl.specification = {
            generations = [
              {
                generator = "text";
                arguments = {
                  inherit name;
                  text = nixosText;
                };
              }
            ];
          };

          home-manager.users.test = {
            sops.secrets.${name} = {
              path = homePath;
            };

            cryl.enable = true;

            cryl.sops.path = "sops/home.yaml";
            cryl.sops.age.path = "age/home.yaml";

            cryl.specification = {
              generations = [
                {
                  generator = "text";
                  arguments = {
                    inherit name;
                    text = homeText;
                  };
                }
              ];
            };
          };
        };

      testScript = ''
        start_all()

        # NOTE: a bit hacky but it works
        machine.wait_for_unit("multi-user.target")
        machine.succeed("""
          sshpass -p ${passShellArg} ssh \
            -o StrictHostKeyChecking=no \
            test@localhost \
            'systemctl --user start --wait sops-nix'
        """)
        machine.succeed("""[ -f ${homePathShellArg} ]""")
        machine.succeed("""[ "$(cat ${homePathShellArg})" = ${homeTextShellArg} ]""")
        machine.succeed("""[ -f ${nixosPathShellArg} ]""")
        machine.succeed("""[ "$(cat ${nixosPathShellArg})" = ${nixosTextShellArg} ]""")
      '';
    };
}
