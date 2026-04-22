{
  cryl.test.modules.multi =
    { lib, ... }:
    let
      makeText = node: "Hello ${node} from cryl!";
      makeTextShellArg = node: lib.escapeShellArg (makeText node);
      name = "cryl-test-text";
      path = "/etc/test/${name}";
      pathShellArg = lib.escapeShellArg path;
      nodes = [
        "machine1"
        "machine2"
        "machine3"
      ];
    in
    {
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
                  generator = "text";
                  arguments = {
                    inherit name;
                    text = makeText node;
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
          ${node}.succeed("""[ "$(cat ${pathShellArg})" = ${makeTextShellArg node} ]""")
        '') nodes
      ));
    };
}
