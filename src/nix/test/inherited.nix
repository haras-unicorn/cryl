{
  cryl.test.modules.inherited =
    { lib, ... }:
    let
      text = "Hello from cryl!";
      textShellArg = lib.escapeShellArg text;
      name = "cryl-test-text";
      home = "/home/test";
      path = "${home}/${name}";
      pathShellArg = lib.escapeShellArg path;
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

          cryl.enable = true;

          cryl.specification = {
            generations = [
              {
                generator = "text";
                arguments = {
                  inherit text name;
                };
              }
            ];
          };

          home-manager.users.test = {
            sops.secrets.${name} = {
              inherit path;
            };

            cryl.enable = true;
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
        machine.succeed("""[ -f ${pathShellArg} ]""")
        machine.succeed("""[ "$(cat ${pathShellArg})" = ${textShellArg} ]""")
      '';
    };
}
