## enable

Whether to enable cryl\.

_Type:_ boolean

_Default:_

```nix
false
```

_Example:_

```nix
true
```

## sops\.age\.export

Whether to export the private SOPS age file

_Type:_ boolean

_Default:_ `true` for tests and `false` for flakes

## sops\.age\.path

Age private key location relative to the root of the output\. For tests relative
to the root of the generated SOPS package and for flakes relative to the flake
root\. Make sure to add this path to gitignore\.

_Type:_ string

## sops\.age\.private

Name of the generated private age key

_Type:_ string

_Default:_

```nix
"age-private"
```

## sops\.age\.public

Name of the generated public age key

_Type:_ string

_Default:_

```nix
"age-public"
```

## sops\.path

Encrypted SOPS file location relative to the root of the output\. For tests
relative to the root of the generated SOPS package and for flakes relative to
the flake root\.

_Type:_ string

## sops\.private

Name of the generated decrypted SOPS file

_Type:_ string

_Default:_

```nix
"sops-private"
```

## sops\.public

Name of the generated encrypted SOPS file

_Type:_ string

_Default:_

```nix
"sops-public"
```

## sops\.secrets

SOPS secrets listing

_Type:_ attribute-tagged union with choices: deep, flat, list, map

_Default:_

```nix
{
  deep = null;
}
```

## sops\.secrets\.deep

Deep listing type

_Type:_ literal value ‘’

_Default:_

```nix
null
```

## sops\.secrets\.flat

Flat listing type

_Type:_ literal value ‘’

_Default:_

```nix
null
```

## sops\.secrets\.list

List listing type

_Type:_ list of absolute path

_Default:_

```nix
[ ]
```

## sops\.secrets\.map

Map listing type

_Type:_ attribute set of absolute path

_Default:_

```nix
{ }
```

## specification

Cryl specification for this nixos configuration or home configuration\.

_Type:_ submodule

_Default:_

```nix
{ }
```

## specification\.exports

Cryl exports specification value\.

_Type:_ list of raw value

_Default:_

```nix
[ ]
```

## specification\.generations

Cryl generations specification value\.

_Type:_ list of raw value

_Default:_

```nix
[ ]
```

## specification\.imports

Cryl imports specification value\.

_Type:_ list of raw value

_Default:_

```nix
[ ]
```
