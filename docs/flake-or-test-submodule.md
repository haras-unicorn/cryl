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

## extraArgs

Extra cryl arguments

_Type:_ list of string

_Default:_

```nix
[ ]
```

## sandboxed

Whether to run cryl sandboxed

_Type:_ boolean

_Default:_ `true` for flakes, `false` for tests

## specifications

Cryl specification attrset

_Type:_ attribute set of (submodule)

_Default:_

```nix
{ }
```

## specifications\.\<name>\.exports

Cryl exports specification value\.

_Type:_ list of raw value

_Default:_

```nix
[ ]
```

## specifications\.\<name>\.generations

Cryl generations specification value\.

_Type:_ list of raw value

_Default:_

```nix
[ ]
```

## specifications\.\<name>\.imports

Cryl imports specification value\.

_Type:_ list of raw value

_Default:_

```nix
[ ]
```
