{
  cryl.test.modules.shared =
    { lib, ... }:
    let
      text = "Hello from cryl!";
      textShellArg = lib.escapeShellArg text;
      name = "cryl-test-text";
      sharedName = "$out/shared/${name}";
      path = "/etc/test/${name}";
      pathShellArg = lib.escapeShellArg path;
      nodes = [
        "machine1"
        "machine2"
        "machine3"
      ];
    in
    {
      cryl.specifications.default = {
        generations = lib.mkBefore [
          {
            generator = "text";
            arguments = {
              inherit text;
              name = sharedName;
            };
          }
        ];
      };

      nodes = lib.listToAttrs (
        builtins.map (node: {
          name = node;
          value = {
            sops.secrets.${name} = {
              inherit path;
            };

            cryl.enable = true;

            cryl.specification = {
              generations = [
                {
                  generator = "copy";
                  arguments = {
                    from = sharedName;
                    to = name;
                  };
                }
              ];
            };
          };
        }) nodes
      );

      testScript = ''
        start_all()
      ''
      + (builtins.concatStringsSep "\n" (
        builtins.map (node: ''
          ${node}.succeed("""[ -f ${pathShellArg} ]""")
          ${node}.succeed("""[ "$(cat ${pathShellArg})" = ${textShellArg} ]""")
        '') nodes
      ));
    };

}
