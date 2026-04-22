{
  cryl.test.modules.nixos =
    { lib, ... }:
    let
      text = "Hello from cryl!";
      textShellArg = lib.escapeShellArg text;
      name = "cryl-test-text";
      path = "/etc/test/${name}";
      pathShellArg = lib.escapeShellArg path;
    in
    {
      nodes.machine = {
        sops.secrets.${name} = {
          inherit path;
        };

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
      };

      testScript = ''
        start_all()

        machine.succeed("""[ -f ${pathShellArg} ]""")
        machine.succeed("""[ "$(cat ${pathShellArg})" = ${textShellArg} ]""")
      '';
    };
}
