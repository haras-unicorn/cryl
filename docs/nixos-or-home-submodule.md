## enable

Whether to enable cryl\.

_Type:_ boolean

_Default:_ `false`

_Example:_ `true`

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

_Default:_ `"age-private"`

## sops\.age\.public

Name of the generated public age key

_Type:_ string

_Default:_ `"age-public"`

## sops\.path

Encrypted SOPS file location relative to the root of the output\. For tests
relative to the root of the generated SOPS package and for flakes relative to
the flake root\.

_Type:_ string

## sops\.private

Name of the generated decrypted SOPS file

_Type:_ string

_Default:_ `"sops-private"`

## sops\.public

Name of the generated encrypted SOPS file

_Type:_ string

_Default:_ `"sops-public"`

## sops\.secrets

SOPS secrets listing

_Type:_ attribute-tagged union

_Default:_

```
{
  deep = null;
}
```

## sops\.secrets\.deep

Deep listing type

_Type:_ literal value ‘’

_Default:_ `null`

## sops\.secrets\.flat

Flat listing type

_Type:_ literal value ‘’

_Default:_ `null`

## sops\.secrets\.list

List listing type

_Type:_ list of absolute path

_Default:_ `[ ]`

## sops\.secrets\.map

Map listing type

_Type:_ attribute set of absolute path

_Default:_ `{ }`

## specification

Cryl specification for this nixos configuration or home configuration\.

_Type:_ submodule

_Default:_ `{ }`

## specification\.exports

Cryl exports specification value\.

_Type:_ list of raw value

_Default:_ `[ ]`

## specification\.generations

Cryl generations specification value\.

_Type:_ list of raw value

_Default:_ `[ ]`

## specification\.imports

Cryl imports specification value\.

_Type:_ list of raw value

_Default:_ `[ ]`
