## enable

Whether to enable cryl\.

_Type:_ boolean

_Default:_ `false`

_Example:_ `true`

## sops\.age\.path

Age private key location relative to the root of the output\. For tests relative
to the root of the generated SOPS package and for flakes relative to the flake
root\. Make sure to add this path to gitignore\.

_Type:_ string

## sops\.path

Encrypted SOPS file location relative to the root of the output\. For tests
relative to the root of the generated SOPS package and for flakes relative to
the flake root\.

_Type:_ string

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
