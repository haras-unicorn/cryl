#!/usr/bin/env -S nu --stdin

###############################################################################
# CONSTANTS
###############################################################################

let main = $"($env.FILE_PWD)/main.nu"

let schema = $"($env.FILE_PWD)/schema.json"

let version = "1.1.3-dev"

let tools = [
  nu
  chmod # NOTE: not a nushell builtin - need from coreutils
  age-keygen
  sops
  nebula-cert
  openssl
  mkpasswd
  mo
  dirname # NOTE: used by mo
  cat # NOTE: used by mo
  ssh-keygen
  wg
  vault
  medusa
  argon2
  ssss-combine
  ssss-split
  cockroach
  bwrap
  prlimit
  systemd-run
]

let tmp_suffix = ".tmp"

let format_suffix = ".format"

let ip_regex = "(^((25[0-5]|(2[0-4]|1\\d|[1-9]|)\\d)\\.?\\b){4}$)"

let tls_algorithm_args = [
  EC
  ec_paramgen_curve:prime256v1
]

let tls_rsa_algorithm_args = [
  RSA
  rsa_keygen_bits:4096
]

let timestamp_format = "%+"

let vault_timestamp_format = "%Y%m%d%H%M%S"

###############################################################################
# TOP-LEVEL INTERFACE
###############################################################################

# Rumor secret generation script
def "main" []: nothing -> nothing {
  nu -c $"($main) -h"
}

# run rumor with specification from path
def "main from-path" [
  # path to specification
  spec: path,
  # stay in current working directory
  # (does nothing unless --nosandbox specified)
  --stay,
  # don't remove the generated secrets
  # (does nothing unless --nosandbox specified)
  --keep,
  # don't run exports
  --dry-run,
  # allow script generator
  --allow-script,
  # maximum allowed imports
  --max-imports: int = 1024,
  # maximum allowed generations
  --max-generations: int = 1024,
  # maximum allowed exports
  --max-exports: int = 1024,
  # maximum allowed specification size in bytes
  --max-specification-size: int = (1024 * 1024),
  # don't use sandbox while running
  --nosandbox,
  # additional read-only bind mounts to add to bubblewrap
  #  - useful for copy imports (does nothing with --nosandbox)
  --ro-binds: list<string> = [],
  # additional bind mounts to add to bubblewrap
  #  - useful for copy exports (does nothing with --nosandbox)
  --binds: list<string> = [],
  # list of tool binaries that rumor is allowed to access via PATH
  # (does nothing with --nosandbox)
  --tools: list<string> = [],
  # allow network while running
  # (does nothing with --nosandbox)
  --allow-net,
  # maximum allowed runtime in seconds
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --timeout: int = (60 * 60),
  # maximum allowed memory while running in bytes
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --max-mem: int = (1024 * 1024 * 128),
  # maximum allowed tasks while running
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --max-tasks: int = 64,
  # maximum allowed generated file size while running in bytes
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --max-file-size: int = (1024 * 1024 * 128),
  # maximum allowed open files while running
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --max-open-files: int = 1024,
  # select manifest format from 'json', 'yaml' and 'toml'
  --manifest-format: string = "json",
  # turn on logging from modules
  --verbose
  # turn on logging from tools (implies verbose)
  --very-verbose
]: nothing -> nothing {
  if $verbose or $very_verbose {
    export-env { $env.__RUMOR_VERBOSE = "1" }
  }
  if $very_verbose {
    export-env { $env.__RUMOR_VERY_VERBOSE = "1" }
  }

  let specification_format = rumor format detect $spec
  if ($specification_format | is-empty) {
    rumor log --error "Unknown specification format."
    exit 1
  }

  let script = open --raw $main
  let schema_string = open --raw $schema
  let commandline = commandline
  let specification_string = open --raw $spec
  let specification = $specification_string
    | rumor format deserialize $specification_format
  let parsed_commandline = {
    spec: $spec
    stay: $stay
    keep: $keep
    dry_run: $dry_run
    allow_script: $allow_script
    max_imports: $max_imports
    max_generations: $max_generations
    max_exports: $max_exports
    max_specification_size: $max_specification_size
    nosandbox: $nosandbox
    ro_binds: $ro_binds
    binds: $binds
    tools: $tools
    allow_net: $allow_net
    timeout: $timeout
    max_mem: $max_mem
    max_tasks: $max_tasks
    max_file_size: $max_file_size
    max_open_files: $max_open_files
    manifest_format: $manifest_format
    verbose: $verbose
    very_verbose: $very_verbose
  }

  (rumor validate
    $script
    $schema_string
    $commandline
    $parsed_commandline
    $specification
    $specification_format
    $specification_string)
}

# run rumor with specification from stdin
def "main from-stdin" [
  # format of the specification
  format: string,
  # stay in current working directory
  # (does nothing unless --nosandbox specified)
  --stay,
  # don't remove the generated secrets
  # (does nothing unless --nosandbox specified)
  --keep,
  # don't run exports
  --dry-run,
  # allow script generator
  --allow-script,
  # maximum allowed imports
  --max-imports: int = 1024,
  # maximum allowed generations
  --max-generations: int = 1024,
  # maximum allowed exports
  --max-exports: int = 1024,
  # maximum allowed specification size in bytes
  --max-specification-size: int = (1024 * 1024),
  # don't use sandbox while running
  --nosandbox,
  # additional read-only bind mounts to add to bubblewrap
  #  - useful for copy imports (does nothing with --nosandbox)
  --ro-binds: list<string> = [],
  # additional bind mounts to add to bubblewrap
  #  - useful for copy exports (does nothing with --nosandbox)
  --binds: list<string> = [],
  # additional list of tool binaries that rumor is allowed to access via PATH
  # (does nothing with --nosandbox)
  --tools: list<string> = [],
  # allow network while running
  # (does nothing with --nosandbox)
  --allow-net,
  # maximum allowed runtime in seconds
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --timeout: int = (1000 * 60 * 60),
  # maximum allowed memory while running in bytes
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --max-mem: int = (1024 * 1024 * 128),
  # maximum allowed tasks while running
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --max-tasks: int = 64,
  # maximum allowed generated file size while running in bytes
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --max-file-size: int = (1024 * 1024 * 128),
  # maximum allowed open files while running
  # (does nothing with --nosandbox)
  # planned: currently no-op
  --max-open-files: int = 1024,
  # select manifest format from 'json', 'yaml' and 'toml'
  --manifest-format: string = "json",
  # turn on logging from modules
  --verbose
  # turn on logging from tools (implies verbose)
  --very-verbose
]: string -> nothing {
  if $verbose or $very_verbose {
    export-env { $env.__RUMOR_VERBOSE = "1" }
  }
  if $very_verbose {
    export-env { $env.__RUMOR_VERY_VERBOSE = "1" }
  }

  let specification_format = $format
  if not (rumor format valid $specification_format) {
    rumor log --error "Unknown specification format."
    exit 1
  }

  let script = open --raw $main
  let schema_string = open --raw $schema
  let commandline = commandline
  let specification_string = $in
  let specification = $specification_string
    | rumor format deserialize $specification_format
  let parsed_commandline = {
    format: $format
    stay: $stay
    keep: $keep
    dry_run: $dry_run
    allow_script: $allow_script
    max_imports: $max_imports
    max_generations: $max_generations
    max_exports: $max_exports
    max_specification_size: $max_specification_size
    nosandbox: $nosandbox
    ro_binds: $ro_binds
    binds: $binds
    tools: $tools
    allow_net: $allow_net
    timeout: $timeout
    max_mem: $max_mem
    max_tasks: $max_tasks
    max_file_size: $max_file_size
    max_open_files: $max_open_files
    manifest_format: $manifest_format
    verbose: $verbose
    very_verbose: $very_verbose
  }

  (rumor validate
    $script
    $schema_string
    $commandline
    $parsed_commandline
    $specification
    $specification_format
    $specification_string)
}

# run rumor from input manifest - USE ONLY FOR DEBUGGING/TESTING!
def "main from-manifest" [
  # path to the input manifest
  input_manifest_path: string
]: nothing -> nothing {
  if ($env.__RUMOR_SANDBOX? | is-not-empty) {
    rumor run $input_manifest_path
    return
  }

  let input_manifest_format = rumor format detect $input_manifest_path
  let input_manifest = rumor format read $input_manifest_path
  let x = $input_manifest.input.parsed_commandline | from json
  let nosandbox = $x.nosandbox
  let verbose = $x.verbose
  let very_verbose = $x.very_verbose

  if $verbose or $very_verbose {
    export-env { $env.__RUMOR_VERBOSE = "1" }
  }
  if $very_verbose {
    export-env { $env.__RUMOR_VERY_VERBOSE = "1" }
  }

  if $nosandbox {
    if not $x.stay {
      cd (rumor mktemp --directory work)
    }
    let work = pwd
    rumor run $input_manifest_path
    if not $x.stay and not ($work | str starts-with "/tmp/rumor-") {
      rumor log --error "Working directory variable got clobbered."
      exit 1
    }
    if not $x.keep and not ((pwd) == $work) {
      rumor log --error "Unexpectedly moved out of target directory."
      exit 1
    }
    if not $x.keep {
      rumor purge workdir
    }
    return
  }

  mut systemd_options = []
  mut bwrap_options = []
  mut prlimit_options = []
  mut nu_options = []

  $bwrap_options ++= [
    --clearenv
    --setenv __RUMOR_SANDBOX "1"
  ]

  if $verbose or $very_verbose {
    $bwrap_options ++= [
      --setenv __RUMOR_VERBOSE "1"
    ]
  }
  if $very_verbose {
    $bwrap_options ++= [
      --setenv __RUMOR_VERY_VERBOSE "1"
    ]
  }

  let manifest_inside = $"/input/manifest.($input_manifest_format)"
  let main_tmp = rumor mktemp --suffix .nu main
  cp -f $main $main_tmp
  let main_inside = $"/input/main.nu"
  $bwrap_options ++= [
    --dir /input
    --ro-bind $input_manifest_path $manifest_inside
    --ro-bind $main_tmp $main_inside
  ]
  $nu_options ++= [
    $main_inside
    from-manifest
    $manifest_inside
  ]

  $bwrap_options ++= [
    --tmpfs /work
    --chdir /work
  ]

  $bwrap_options ++= ($x.ro_binds
    | each {
        let absolute = realpath $in
        [ --ro-bind $absolute $absolute ]
      }
    | flatten)
  $systemd_options ++= ($x.ro_binds
    | each {
        let absolute = realpath $in
        [ -p ReadOnlyPaths=($absolute) ]
      }
    | flatten)
  $systemd_options ++= ($x.binds
    | each {
        let absolute = realpath $in
        [ -p ReadWritePaths=($absolute) ]
      }
    | flatten)
  $bwrap_options ++= ($x.binds
    | each {
        let absolute = realpath $in
        [ --bind $absolute $absolute ]
      }
    | flatten)

  let tool_path = rumor mktemp --directory tools
  ($x.tools ++ $tools) | each {
    let absolute = realpath (which $in).0.path
    let link = [ $tool_path $in ] | path join
    ln -s $absolute $link
  }
  $bwrap_options ++= [ --ro-bind $tool_path /tools ]
  $bwrap_options ++= [
    --setenv PATH /tools
  ]
  if ("/nix/store" | path exists) {
    $bwrap_options ++= [ --ro-bind /nix /nix ]
  } else {
    if ("/usr" | path exists) {
      $bwrap_options ++= [ --ro-bind /usr /usr ]
    }
    if ("/bin" | path exists) {
      $bwrap_options ++= [ --ro-bind /bin /bin ]
    }
    if ("/lib" | path exists) {
      $bwrap_options ++= [ --ro-bind /lib /lib ]
    }
    if ("/lib64" | path exists) {
      $bwrap_options ++= [ --ro-bind /lib64 /lib64 ]
    }
  }
  if ("/etc" | path exists) {
    $bwrap_options ++= [ --ro-bind /etc /etc ]
  }
  $bwrap_options ++= [
    --setenv LC_ALL C.UTF-8
    --setenv LANG C.UTF-8
  ]
  $bwrap_options ++= [
    --tmpfs /tmp
    --setenv TMPDIR /tmp
  ]
  $systemd_options ++= [
    -p ProtectHome=tmpfs
  ]
  $bwrap_options ++= [
    --dir /home
    --setenv HOME /home
  ]

  $systemd_options ++= [
    -p RuntimeMaxSec=($x.timeout)
    -p MemoryMax=($x.max_mem)
    -p TasksMax=($x.max_tasks)
  ]
  if not $x.allow_net {
    $systemd_options ++= [ -p RestrictAddressFamilies=AF_UNIX ]
    $bwrap_options ++= [ --unshare-net ]
  }
  $systemd_options ++= [
    -p PrivateTmp=yes
    -p ProtectSystem=strict
    -p PrivateDevices=yes
    -p NoNewPrivileges=yes
    -p LockPersonality=yes
    -p RestrictNamespaces=yes
    -p ProtectKernelTunables=yes
    -p ProtectKernelModules=yes
    -p ProtectControlGroups=yes
  ]
  $bwrap_options ++= [
    --die-with-parent
    --unshare-user
    --uid 0
    --gid 0
    --unshare-pid
    --unshare-uts
    --unshare-ipc
    --proc /proc
    --dev-bind /dev /dev
  ]
  $prlimit_options ++= [
    --fsize=($x.max_file_size)
    --nofile=($x.max_open_files)
  ]

  let result = try {
    # TODO: too much privilege nonsense
    # systemd-run --user --wait --pipe --pty ...($systemd_options)
    # TODO: cant find bwrap...
    # prlimit ...($prlimit_options) --
    (bwrap ...($bwrap_options) --
      nu ...($nu_options))
  } | complete
  rm -rf $tool_path
  rm -rf $main_tmp

  if $result.exit_code != 0 {
    let message = ($"sandbox failed and"
      + $" exited with '($result.exit_code)'")
    $result.stderr | rumor log --error "sandbox failed"
    error make {
      msg: $"sandbox failed with exit code ($result.exit_code)"
    }
  }
}

###############################################################################
# IMPORTERS
###############################################################################

# copy a file from one path to another
def "main import copy" [
  # from where to copy the file
  from: path,
  # where to put the file
  to: path,
  # allow failing to copy if source missing
  --allow-fail,
  # overwrite destination if it exists
  --renew
]: nothing -> nothing {
  let content = if $allow_fail and not ($from | path exists) {
    return
  } else {
    open --raw $from
  }

  $content | rumor save $to $renew
}

# import files from a medusa vault path
def "main import vault" [
  # vault path to export from
  path: string,
  # allow failing to import if source missing or command fails
  --allow-fail,
  # overwrite destination files if they exist
  --renew
]: nothing -> nothing {
  let trimmed_path = $path | str trim --char '/'

  let components = $trimmed_path
    | split row "/"
    | skip 1
    | into cell-path

  let result = if $allow_fail {
      try {
        (rumor exec tool "import vault"
          medusa export $trimmed_path)
      } catch {
        return
      }
    } else {
      (rumor exec tool "import vault"
        medusa export $path)
    } | rumor decode if bytes

  let files = $result
    | from yaml
    | get $components
    | get current
    | transpose name value
  for file in $files {
    $file.value | rumor save $file.name $renew
  }
}

# import a single file from a vault path
def "main import vault-file" [
  # vault path to load from
  path: string,
  # file key to extract
  file: string,
  # allow failing to load if source missing or command fails
  --allow-fail,
  # overwrite destination file if it exists
  --renew
]: nothing -> nothing {
  let trimmed_path = $path | str trim --char '/'

  let result = if $allow_fail {
      try {
        (rumor exec tool "import vault-file"
          vault kv get -format=json $"($trimmed_path)/current")
      } catch {
        return
      }
    } else {
      (rumor exec tool "import vault-file"
        vault kv get -format=json $"($trimmed_path)/current")
    } | rumor decode if bytes

  if $allow_fail {
    try {
      $result
        | from json
        | get data.data
        | get $file
    } catch {
      return
    }
  } else {
    $result
      | from json
      | get data.data
      | get $file
  } | rumor save $file $renew
}

###############################################################################
# GENERATORS
###############################################################################

# copy a file as part of generation
def "main generate copy" [
  # source file path
  from: path,
  # destination file path
  to: path,
  # overwrite destination if it exists
  --renew
]: nothing -> nothing {
  open --raw $from | rumor save $to $renew
}

# write a text file as part of generation
def "main generate text" [
  # destination file name
  name: path,
  # text content to write
  text: string,
  # overwrite destination if it exists
  --renew
]: nothing -> nothing {
  $text | rumor save $name $renew
}

# generate a data file by converting between formats
def "main generate data" [
  # destination file name
  name: path,
  # input data format
  in_format: string,
  # source data path
  data: path,
  # output data format
  out_format: string,
  # overwrite destination if it exists
  --renew
]: nothing -> nothing {
  if not (rumor format valid $in_format) {
    (rumor log
      --module "generate data"
      --error $"Invalid in format: '($in_format)'")
    exit 1
  }

  if not (rumor format valid $out_format) {
    (rumor log
      --module "generate data"
      --error $"Invalid out format: '($out_format)'")
    exit 1
  }

  if ($renew) {
     rumor format read $data $in_format
      | rumor format write $name $out_format --renew
  } else {
     rumor format read $data $in_format
      | rumor format write $name $out_format
  }
  $out_format | rumor save $"($name)($format_suffix)" $renew
}

# generate a numeric PIN and save it
def "main generate pin" [
  # destination file name
  name: path,
  # number of digits in the PIN
  --length: int = 8
  # overwrite destination if it exists
  --renew
]: nothing -> nothing {
  let pin = rumor secure random digits "generate pin" $length
  $pin | rumor save $name $renew
}

# generate a random alphanumeric key and save it
def "main generate key" [
  # destination file name
  name: path,
  # number of characters in the key
  --length: int = 32,
  # overwrite destination if it exists
  --renew
]: nothing -> nothing {
  let id = rumor secure random alnum "generate key" $length
  $id | rumor save $name $renew
}

# generate a random alphanumeric id and save it
def "main generate id" [
  # destination file name
  name: path,
  # number of characters in the id
  --length: int = 16,
  # overwrite destination if it exists
  --renew
]: nothing -> nothing  {
  let id = rumor secure random alnum "generate id" $length
  $id | rumor save $name $renew
}

# generate a random password, save plaintext + hashed
def "main generate password" [
  # path to save the hashed (public) password
  public: path,
  # path to save the plaintext (private) password
  private: path,
  # number of characters in the password
  --length: int = 8,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  let pass = rumor secure random alnum "generate password" $length
  let salt = openssl rand -base64 32
  let encrypted =  $pass
    | (rumor exec tool "generate password"
        argon2 $salt -e -id -k 19456 -t 2 -p 1)

  $pass | rumor save $private $renew
  $encrypted | rumor save $public $renew --public
}

# generate a random password, save plaintext + yescrypt hash
def "main generate password-crypt-3" [
  # path to save the hashed (public) password
  public: path,
  # path to save the plaintext (private) password
  private: path,
  # number of characters in the password
  --length: int = 8,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  let pass = (rumor secure random alnum
    "generate password-crypt-3"
    $length)
  let encrypted = $pass
    | (rumor exec tool "generate password-crypt-3"
        mkpasswd --stdin --method=yescrypt)

  $pass | rumor save $private $renew
  $encrypted | rumor save $public $renew --public
}

# generate an age key pair and save public + private
def "main generate age-key" [
  # path to save the public key (string path is fine)
  public: string,
  # path to save the private key
  private: string,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  let private_content = (rumor exec tool "generate age-key"
    age-keygen)
  let public_content = $private_content
    | (rumor exec tool "generate age-key"
        age-keygen -y)

  $private_content | rumor save $private $renew
  $public_content | rumor save $public $renew --public
}

# generate an SSH key pair and save public + private
def "main generate ssh-key" [
  # key comment (e.g., email or host)
  name: string,
  # path to save the public key
  public: path,
  # path to save the private key
  private: path,
  # passphrase file path (empty for no passphrase)
  --password: string = "",
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  mut password = $password

  let password_args = if ($password | str trim | is-not-empty) {
    [ -N (open --raw $password) ]
  } else {
    [ -N "''" ]
  }

  (rumor exec tool "generate ssh-key"
    ssh-keygen
      -a 100
      -t ed25519
      -C $name
      ...($password_args)
      -f $"($private)($tmp_suffix)")
  let private_content = open --raw  $"($private)($tmp_suffix)"
  let public_content = open --raw  $"($private)($tmp_suffix).pub"
  rm -f $"($private)($tmp_suffix)"
  rm -f $"($private)($tmp_suffix).pub"

  $private_content | rumor save $private $renew
  $public_content | rumor save $public $renew --public
}

# generate a WireGuard key pair and save public + private
def "main generate wireguard-key" [
  # path to save the private key
  private: path,
  # path to save the public key
  public: path,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  let private_content = (rumor exec tool
    "generate wireguard-private"
    wg genkey)

  let public_content = ($private_content
    | rumor exec tool
        "generate wireguard-public"
        wg pubkey)

  $private_content | rumor save $private $renew
  $public_content | rumor save $public $renew --public
}

# split a key into Shamir Shares and save them
def "main generate key-split" [
  # path to the source key file (raw content is split)
  key: string,
  # filename prefix for each generated share
  prefix: string,
  # minimum number of shares required to reconstruct
  threshold: int,
  # total number of shares to generate
  shares: int,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  let shares = open --raw $key
    | (rumor exec tool "generate key-split"
        ssss-split
          -t ($threshold | into string)
          -n ($shares | into string)
          -q)
    | lines

  for item in ($shares | enumerate) {
    let share = $"($prefix)-($item.index)"
    $item.item | rumor save $share $renew
  }
}

# combine Shamir Shares back into a single key
def "main generate key-combine" [
  # comma-separated list of share file paths (e.g., "share-0,share-1,share-3")
  shares: string,
  # path to save the reconstructed key
  key: string,
  # number of shares required to reconstruct (must match split threshold)
  threshold: int,
  # overwrite destination if it exists
  --renew
]: nothing -> nothing {
  let shares = $shares
    | split row ","
    | each { open --raw }
    | str join "\n"
  let value = $"($shares)\n"
    | (rumor exec tool "generate key-combine"
        ssss-combine
          -t ($threshold | into string)
          -q)
  $value | rumor save $key $renew
}

# generate a TLS Root CA (private key + self-signed cert)
def "main generate tls-root" [
  # Common Name for the Root CA (e.g., "Sarah Root CA")
  common_name: string,
  # Organization (e.g., "Green Energy Devs")
  organization: string,
  # path to write the openssl config it generates
  config: path,
  # path to save the private key
  private: path,
  # path to save the root certificate (public)
  public: path,
  # allowed intermediate depth (use -1 for unlimited)
  --pathlen: int = 1,
  # certificate validity in days
  --days: int = 3650,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  (rumor tls root
    "generate tls-root"
    $tls_algorithm_args.0
    $tls_algorithm_args.1
    $common_name
    $organization
    $config
    $private
    $public
    $pathlen
    $days
    $renew)
}

# issue an Intermediate CA (key + CSR + signed cert)
def "main generate tls-intermediary" [
  # Common Name for the Intermediate CA
  common_name: string,
  # Organization
  organization: string,
  # path to write the merged OpenSSL config (ext + req)
  config: path,
  # path to save the intermediate private key
  private: path,
  # path to save the CSR
  request: path,
  # path to read base request config (will be extended)
  request_config: path,
  # Root CA cert (public)
  ca_public: path,
  # Root CA key (private)
  ca_private: path,
  # serial file to track issued cert serials
  serial: path,
  # path to save the signed intermediate cert (public)
  public: path,
  # allowed subordinate depth (use -1 for unlimited)
  --pathlen: int = 0,
  # certificate validity in days
  --days: int = 3650,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  (rumor tls intermediary
    "generate tls-intermediary"
    $tls_algorithm_args.0
    $tls_algorithm_args.1
    $common_name
    $organization
    $config
    $request_config
    $private
    $request
    $ca_public
    $ca_private
    $serial
    $public
    $pathlen
    $days
    $renew)
}

# issue a Leaf cert (key + CSR + signed cert)
def "main generate tls-leaf" [
  # Common Name for the certificate
  common_name: string,
  # Organization
  organization: string,
  # comma-separated SANs (e.g., "example.com,www.example.com,10.0.0.1")
  sans: string,
  # path to write the final OpenSSL ext config
  config: path,
  # path to write the CSR req config (will be created)
  request_config: path,
  # path to save the private key
  private: path,
  # path to save the CSR
  request: path,
  # Issuer cert (public)
  ca_public: path,
  # Issuer key (private)
  ca_private: path,
  # serial file to track issued cert serials
  serial: path,
  # path to save the signed cert (public)
  public: path,
  # validity in days
  --days: int = 3650,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  (rumor tls leaf
    "generate tls-leaf"
    $tls_algorithm_args.0
    $tls_algorithm_args.1
    $common_name
    $organization
    $sans
    $config
    $request_config
    $private
    $request
    $ca_public
    $ca_private
    $serial
    $public
    $days
    $renew)
}

# issue a ROOT CA (key + self-signed cert)
def "main generate tls-rsa-root" [
  # Common Name for the Root CA
  common_name: string,
  # Organization
  organization: string,
  # path to write the OpenSSL ext config
  config: path,
  # path to save the root private key
  private: path,
  # path to save the self-signed root cert (public)
  public: path,
  # allowed subordinate depth (use -1 for unlimited)
  --pathlen: int = 1,
  # certificate validity in days
  --days: int = 3650,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  (rumor tls root
    "generate tls-root"
    $tls_rsa_algorithm_args.0
    $tls_rsa_algorithm_args.1
    $common_name
    $organization
    $config
    $private
    $public
    $pathlen
    $days
    $renew)
}

# issue a RSA Intermediate CA (key + CSR + signed cert)
def "main generate tls-rsa-intermediary" [
  # Common Name for the Intermediate CA
  common_name: string,
  # Organization
  organization: string,
  # path to write the merged OpenSSL config (ext + req)
  config: path,
  # path to write the request config (will be created/overwritten)
  request_config: path,
  # path to save the Intermediate private key
  private: path,
  # path to save the CSR
  request: path,
  # Root/Issuer cert (public)
  ca_public: path,
  # Root/Issuer key (private)
  ca_private: path,
  # serial file to track issued cert serials
  serial: path,
  # path to save the signed intermediate cert (public)
  public: path,
  # allowed subordinate depth (use -1 for unlimited)
  --pathlen: int = 0,
  # certificate validity in days
  --days: int = 3650,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  (rumor tls intermediary
    "generate tls-intermediary"
    $tls_rsa_algorithm_args.0
    $tls_rsa_algorithm_args.1
    $common_name
    $organization
    $config
    $request_config
    $private
    $request
    $ca_public
    $ca_private
    $serial
    $public
    $pathlen
    $days
    $renew)
}

# issue a RSA Leaf cert (key + CSR + signed cert)
def "main generate tls-rsa-leaf" [
  # Common Name for the certificate (e.g., domain)
  common_name: string,
  # Organization
  organization: string,
  # Subject Alternative Names, comma-separated (e.g., "example.com,www.example.com,10.0.0.1")
  sans: string,
  # path to write the merged OpenSSL config (ext + req)
  config: path,
  # path to write the request config (will be created/overwritten)
  request_config: path,
  # path to save the private key
  private: path,
  # path to save the CSR
  request: path,
  # Issuer cert (Intermediate or Root)
  ca_public: path,
  # Issuer key (matching private key)
  ca_private: path,
  # serial file to track issued cert serials
  serial: path,
  # path to save the signed leaf certificate (public)
  public: path,
  # certificate validity in days
  --days: int = 3650,
  # overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  (rumor tls leaf
    "generate tls-leaf"
    $tls_rsa_algorithm_args.0
    $tls_rsa_algorithm_args.1
    $common_name
    $organization
    $sans
    $config
    $request_config
    $private
    $request
    $ca_public
    $ca_private
    $serial
    $public
    $days
    $renew)
}

# generate OpenSSL Diffie-Hellman parameters (dhparam)
def "main generate tls-dhparam" [
  # Path to save the DH parameters file
  name: path,
  # Overwrite destination if it exists
  --renew
]: nothing -> nothing {
  let dhparam = (rumor exec tool "generate tls-dhparam"
    openssl dhparam -quiet 2048)
  $dhparam | rumor save $name $renew
}

# generate a Nebula CA (certificate + key)
def "main generate nebula-ca" [
  # Common name for the Nebula CA
  name: string,
  # Path to save the CA certificate
  public: path,
  # Path to save the CA private key
  private: path,
  # Certificate validity in days
  --days: int = 3650,
  # Overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  (rumor exec tool "generate nebula-ca"
    nebula-cert ca
    -name $name
    -duration $"($days * 24)h"
    -out-crt $"($public)($tmp_suffix)"
    -out-key $"($private)($tmp_suffix)")
  let public_content = open --raw $"($public)($tmp_suffix)"
  let private_content = open --raw $"($private)($tmp_suffix)"
  rm -f $"($public)($tmp_suffix)"
  rm -f $"($private)($tmp_suffix)"

  $public_content | rumor save $public $renew
  $private_content | rumor save $private $renew
}

# generate a Nebula node certificate (signed by a Nebula CA)
def "main generate nebula-cert" [
  # Path to the Nebula CA certificate (public)
  ca_public: path,
  # Path to the Nebula CA private key
  ca_private: path,
  # Common name for the node cert
  name: string,
  # Node IP in CIDR or IP form (e.g., "10.1.1.5/24" or "10.1.1.5")
  ip: string,
  # Output path for the node certificate
  public: path,
  # Output path for the node private key
  private: path,
  # Overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  (rumor exec tool "generate nebula-cert"
    nebula-cert sign
    -ca-crt $ca_public
    -ca-key $ca_private
    -name $name
    -ip $ip
    -out-crt $"($public)($tmp_suffix)"
    -out-key $"($private)($tmp_suffix)")
  let public_content = open --raw $"($public)($tmp_suffix)"
  let private_content = open --raw $"($private)($tmp_suffix)"
  rm -f $"($public)($tmp_suffix)"
  rm -f $"($private)($tmp_suffix)"

  $public_content | rumor save $public $renew
  $private_content | rumor save $private $renew
}

# generate a CockroachDB CA (certificate + key)
def "main generate cockroach-ca" [
  # Output path for the CA certificate
  public: path,
  # Output path for the CA private key
  private: path,
  # Overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  rm -rf $"cockroach($tmp_suffix)"
  mkdir $"cockroach($tmp_suffix)"
  (rumor exec tool "generate cockroach-ca"
    cockroach cert create-ca
    $"--certs-dir=cockroach($tmp_suffix)"
    $"--ca-key=cockroach($tmp_suffix)/ca.key")
  let public_content = open --raw $"cockroach($tmp_suffix)/ca.crt"
  let private_content = open --raw $"cockroach($tmp_suffix)/ca.key"
  rm -rf $"cockroach($tmp_suffix)"

  $public_content | rumor save $public $renew
  $private_content | rumor save $private $renew
}

# generate a CockroachDB node certificate (signed by CockroachDB CA)
def "main generate cockroach-node-cert" [
  # Path to the CockroachDB CA certificate
  ca_public: path,
  # Path to the CockroachDB CA private key
  ca_private: path,
  # Output path for the node certificate
  public: path,
  # Output path for the node private key
  private: path,
  # Comma-separated host names/IPs for the node cert SANs
  hosts: string,
  # Overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  rm -rf $"cockroach($tmp_suffix)"
  mkdir $"cockroach($tmp_suffix)"
  cp $ca_private $"cockroach($tmp_suffix)/ca.key"
  cp $ca_public $"cockroach($tmp_suffix)/ca.crt"
  (rumor exec tool "generate cockroach-node-cert"
    cockroach cert create-node
    ...($hosts | str trim | split row ",")
    $"--certs-dir=cockroach($tmp_suffix)"
    $"--ca-key=cockroach($tmp_suffix)/ca.key")
  let public_content = open --raw $"cockroach($tmp_suffix)/node.crt"
  let private_content = open --raw $"cockroach($tmp_suffix)/node.key"
  rm -rf $"cockroach($tmp_suffix)"

  $public_content | rumor save $public $renew
  $private_content | rumor save $private $renew
}

# generate a CockroachDB client certificate (for a specific user)
def "main generate cockroach-client-cert" [
  # Path to the CockroachDB CA certificate
  ca_public: path,
  # Path to the CockroachDB CA private key
  ca_private: path,
  # Output path for the client certificate
  public: path,
  # Output path for the client private key
  private: path,
  # CockroachDB username for the client cert
  user: string,
  # Overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  rm -rf $"cockroach($tmp_suffix)"
  mkdir $"cockroach($tmp_suffix)"
  cp $ca_private $"cockroach($tmp_suffix)/ca.key"
  cp $ca_public $"cockroach($tmp_suffix)/ca.crt"
  (rumor exec tool "generate cockroach-client-cert"
    cockroach cert create-client
    $user
    $"--certs-dir=cockroach($tmp_suffix)"
    $"--ca-key=cockroach($tmp_suffix)/ca.key")
  let public_content = (open
    --raw $"cockroach($tmp_suffix)/client.($user).crt")
  let private_content = (open
    --raw $"cockroach($tmp_suffix)/client.($user).key")
  rm -rf $"cockroach($tmp_suffix)"

  $public_content | rumor save $public $renew
  $private_content | rumor save $private $renew
}

# generate an environment (.env-style) file
def "main generate env" [
  # Where to save the environment file
  name: string,
  # Input format of `vars` (e.g., "json", "yaml", "toml")
  format: string,
  # Variables as a serialized string in the given format
  vars: string,
  # Overwrite destination if it exists
  --renew
]: nothing -> nothing {
  let vars = rumor format read $vars $format

  let vars = $vars
    | transpose key value
    | each { |pair|
        let raw = if ($pair.value | path exists) {
          open --raw $pair.value
        } else {
          $pair.value
        }
        let value = $raw
          | str trim
          | str replace -a "\\" "\\\\"
          | str replace -a "\n" "\\n"
          | str replace -a "\"" "\\\""
        {
          key: $pair.key,
          value: $value
        }
      }
    | reduce --fold "" { |item, accumulator|
        $"($accumulator)\n($item.key)=\"($item.value)\""
      }
    | str trim

  $vars | rumor save $name $renew
}

# generate a populated Mustache template
def "main generate moustache" [
  # Base name where outputs are saved
  name: string,
  # Input format of the combined file ("json", "yaml", "toml")
  format: string,
  # Path to a file containing { template, variables }
  variables_and_template: path,
  # Overwrite destination(s) if they exist
  --renew
]: nothing -> nothing {
  let variables_and_template = (rumor format read
    $variables_and_template
    $format)

  let vars = $variables_and_template.variables
    | transpose key value
    | each { |pair|
        let raw = if ($pair.value | path exists) {
          open --raw $pair.value
        } else {
          $pair.value
        }
        {
          key: $pair.key,
          value: ($raw | rumor escape env string)
        }
      }
    | reduce --fold "" { |item, accumulator|
        $"($accumulator)($item.key)=\"($item.value)\"\n"
      }
    | str trim

  $vars
    | str trim
    | rumor save $"($name)-variables" $renew

  $variables_and_template.template
    | str trim
    | rumor save $"($name)-template" $renew

  (rumor exec tool "generate moustache"
    mo $"--source=($name)-variables" $"($name)-template")
    | rumor save $name $renew
}

# generate and run a Nushell script
def "main generate script" [
  # Where to save the script
  name: string,
  # Script contents
  text: string,
  # Overwrite destination if it exists
  --renew
]: nothing -> nothing {
  $text | rumor save $name $renew
  (rumor exec tool "generate script"
    nu $name $renew)
}

# generate SOPS-encrypted secrets from key-value inputs
def "main generate sops" [
  # Path to a file containing the Age recipient(s)
  age: string,
  # Where to save the encrypted secrets (SOPS YAML)
  public: string,
  # Where to save the plaintext secrets (YAML)
  private: string,
  # Input format for `secrets` ("json", "yaml", "toml")
  format: string,
  # Path to a file containing the secrets object (values or file paths)
  values: path,
  # Overwrite destinations if they exist
  --renew
]: nothing -> nothing {
  let values = rumor format read $values $format

  let values = if ($values == {}) {
    {} | to yaml
  } else { $values
    | transpose key value
    | each { |secret|
        let raw = if ($secret.value | path exists) {
          open --raw $secret.value
        } else {
          $secret.value
        }
        let value = $raw | str trim
        {
          key: $secret.key,
          value: $value
        }
      }
    | transpose -r -d --ignore-titles
    | to yaml
  }

  $values | rumor save $private $renew

  let encrypted = (rumor exec tool "generate sops"
    sops encrypt $private
    --input-type yaml
    --age (open --raw $age | str trim)
    --output-type yaml)

  $encrypted | rumor save $public $renew --public
}

###############################################################################
# EXPORTERS
###############################################################################

# copy a file from A to B
def "main export copy" [
  # Source file path
  from: path,
  # Destination file path
  to: path
]: nothing -> nothing {
  cp -f $from $to
}

# export current directory as a Vault KV snapshot (current + timestamped)
def "main export vault" [
  # Base vault path (e.g., "kv/my-app")
  path: string
]: nothing -> nothing {
  let trimmed_path = $path | str trim --char '/'

  let files = ls
    | each { |file|
        {
          name: ($file.name | path basename),
          value: (open --raw $file.name | str trim)
        }
      }
    | transpose -r -d --ignore-titles
    | to yaml

  let path = $trimmed_path + "/current"
  $files | (rumor exec tool "export vault"
    medusa import $path -)

  let time = rumor vault now
  let timestamped_path = $"($trimmed_path)/($time)"
  $files | (rumor exec tool "export vault"
    medusa import $timestamped_path -)
}

# export a single file into Vault KV (current + timestamped), patching if exists
def "main export vault-file" [
  # Base vault path (e.g., "kv/my-app")
  path: string,
  # Local file path to upload (key will be the filename you pass)
  file: string
]: nothing -> nothing {
  let trimmed_path = $path | str trim --char '/'
  let path = $trimmed_path + "/current"

  let content = open --raw $file | str trim

  let new = try {
    (rumor exec tool "export vault-file"
      vault kv get -format=json $path)
      | rumor decode if bytes
      | from json
      | get data
      | get data
      | upsert $file $content
    } catch {
      null
    }

  let time = rumor vault now
  let timestamped_path = $"($trimmed_path)/($time)"
  if $new == null {
    $content | (rumor exec tool "export vault-file"
      vault kv put $path $"($file)=-")
    $content | (rumor exec tool "export vault-file"
      vault kv put $timestamped_path $"($file)=-")
  } else {
    $content | (rumor exec tool "export vault-file"
      vault kv patch $path $"($file)=-")
    $new | to yaml | (rumor exec tool "export vault-file"
      medusa import $timestamped_path -)
  }
}

###############################################################################
# TOP-LEVEL IMPLEMENTATION
###############################################################################

def "rumor validate" [
  script: string,
  schema_string: string,
  commandline: string,
  parsed_commandline: any,
  specification: any,
  specification_format: string,
  specification_string: string
]: nothing -> nothing {
  let x = $parsed_commandline

  if (($specification_string | into binary | length)
    > $x.max_specification_size) {
    rumor log --error "Maximum specification size exceeded."
    exit 1
  }

  if ($specification.imports | length) > $x.max_imports {
    rumor log --error "Maximum imports exceeded."
    exit 1
  }

  if ($specification.generations | length) > $x.max_generations {
    rumor log --error "Maximum generations exceeded."
    exit 1
  }

  if ($specification.exports | length) > $x.max_exports {
    rumor log --error "Maximum exports exceeded."
    exit 1
  }

  let validation_result = try {
   $specification
      | to json
      | json-schema-validate $schema
  } | complete
  if ($validation_result.exit_code != 0) {
    $validation_result.stderr
      | rumor log --error $"Specification schema invalid"
    exit 1
  }

  if not (rumor format valid $x.manifest_format) {
    rumor log --error ("Invalid manifest format."
      + " Supported formats are: 'json', 'yaml', 'toml'.")
    exit 1
  }

  let phase = "input"
  let with_output = false
  let manifest = (rumor create manifest
    $script
    $schema_string
    $commandline
    $parsed_commandline
    $specification_string
    $specification_format
    $phase
    null
    $with_output
    $x.manifest_format)

  let manifest_path = (rumor mktemp
    --suffix $".($x.manifest_format)" manifest)
  $manifest | (rumor format write
    $manifest_path
    $x.manifest_format
    --renew
    --public)
  # TODO: cleanup without clobbering error
  main from-manifest $manifest_path
  rm -f $manifest_path
  rm -f $"($manifest_path)($format_suffix)"
}

def "rumor run" [
  input_manifest_path: string
]: nothing -> nothing {
  let input_manifest_format = (rumor format detect
    $input_manifest_path)
  let input_manifest = (rumor format read
    $input_manifest_path)
  let script = $input_manifest.input.script_text
  let schema_string = $input_manifest.input.schema_text
  let commandline = $input_manifest.input.commandline
  let parsed_commandline = (
    $input_manifest.input.parsed_commandline | from json)
  let specification_format = (
    $input_manifest.input.specification_format)
  let specification_string = (
    $input_manifest.input.specification_text)
  let specification = $specification_string
    | rumor format deserialize $specification_format
  let manifest_format = $input_manifest.format
  let dry_run = $parsed_commandline.dry_run
  let verbose = $parsed_commandline.verbose
  let allow_script = $parsed_commandline.allow_script
  let with_output = true

  mut import_error = null
  for import in $specification.imports {
    $import_error = try {
      rumor run import $import
      null
    } catch { |err|
      $err
    }

    if $import_error != null {
      break
    }
  }

  let phase = "import"
  let import_manifest = (rumor create manifest
    $script
    $schema_string
    $commandline
    $parsed_commandline
    $specification_string
    $specification_format
    $phase
    $import_error
    $with_output
    $manifest_format)

  if $import_error != null {
    ($input_manifest | rumor format write
      $"rumor-input-manifest.($manifest_format)"
      $manifest_format
      --renew
      --public)
    ($import_manifest | rumor format write
      $"rumor-import-manifest.($manifest_format)"
      $manifest_format
      --renew
      --public)
    return
  }

  mut generation_error = null
  for generation in $specification.generations {
    $generation_error = try {
      rumor run generation $generation $allow_script
      null
    } catch { |err|
      $err
    }

    if $generation_error != null {
      break
    }
  }

  let phase = "generation"
  let generation_manifest = (rumor create manifest
    $script
    $schema_string
    $commandline
    $parsed_commandline
    $specification_string
    $specification_format
    $phase
    $generation_error
    $with_output
    $manifest_format)

  ($input_manifest | rumor format write
    $"rumor-input-manifest.($manifest_format)"
    $manifest_format
    --renew
    --public)
  ($import_manifest | rumor format write
    $"rumor-import-manifest.($manifest_format)"
    $manifest_format
    --renew
    --public)
  ($generation_manifest | rumor format write
    $"rumor-generation-manifest.($manifest_format)"
    $manifest_format
    --renew
    --public)

  if $generation_error != null or $dry_run {
    return
  }

  mut export_error = null
  for export in $specification.exports {
    $export_error = try {
      rumor run export $export
      null
    } catch { |err|
      $err
    }

    if $export_error != null {
      break
    }
  }
  if $export_error != null {
    return
  }
}

def "rumor run import" [import: any]: nothing -> nothing {
  mut args = [ ]

  if ($import.importer == "vault") {
    $args ++= [ ($import.arguments.path) ]
    if (($import.arguments
      | get -o allow_fail) != null
      and $import.arguments.allow_fail) {
      $args ++= [ --allow-fail ]
    }
  } else if ($import.importer == "vault-file") {
    $args ++= [ ($import.arguments.path) ]
    $args ++= [ ($import.arguments.file) ]
    if (($import.arguments
      | get -o allow_fail) != null
      and $import.arguments.allow_fail) {
      $args ++= [ --allow-fail ]
    }
  } else if ($import.importer == "copy") {
    $args ++= [ ($import.arguments.from) ]
    $args ++= [ ($import.arguments.to) ]
    if (($import.arguments
      | get -o allow_fail) != null
      and $import.arguments.allow_fail) {
      $args ++= [ --allow-fail ]
    }
  }

  rumor exec module $"import ($import.importer)" ...($args)
}

def "rumor run generation" [
  generation: any,
  allow_script: bool
]: nothing -> nothing {
  mut args = [ ]
  mut generator = $generation.generator

  if $generation.generator == "copy" {
    $args ++= [ ($generation.arguments.from) ]
    $args ++= [ ($generation.arguments.to) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "text" {
    $args ++= [ ($generation.arguments.name) ]
    $args ++= [ ($generation.arguments.text | rumor escape arg string) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "json" {
    $generator = "data"
    $args ++= [ ($generation.arguments.name) ]
    let json = $"($generation.arguments.name)-json"
    $args ++= [ json ]
    $args ++= [ ($json) ]
    $args ++= [ json ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
      $generation.arguments.value
        | rumor format write $json json --renew --public
    } else {
      $generation.arguments.value
        | rumor format write $json json --public
    }
  } else if $generation.generator == "yaml" {
    $generator = "data"
    $args ++= [ ($generation.arguments.name) ]
    let yaml = $"($generation.arguments.name)-yaml"
    $args ++= [ yaml ]
    $args ++= [ ($yaml) ]
    $args ++= [ yaml ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
      $generation.arguments.value
        | rumor format write $yaml yaml --renew --public
    } else {
      $generation.arguments.value
        | rumor format write $yaml yaml --public
    }
  } else if $generation.generator == "toml" {
    $generator = "data"
    $args ++= [ ($generation.arguments.name) ]
    let toml = $"($generation.arguments.name)-toml"
    $args ++= [ toml ]
    $args ++= [ ($toml) ]
    $args ++= [ toml ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
      $generation.arguments.value
        | rumor format write $toml toml --renew --public
    } else {
      $generation.arguments.value
        | rumor format write $toml toml --public
    }
  } else if $generation.generator == "id" {
    $args ++= [ ($generation.arguments.name) ]
    if (($generation.arguments | get -o length) != null) {
      $args ++= [ --length ($generation.arguments.length) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "key" {
    $args ++= [ ($generation.arguments.name) ]
    if (($generation.arguments | get -o length) != null) {
      $args ++= [ --length ($generation.arguments.length) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "pin" {
    $args ++= [ ($generation.arguments.name) ]
    if (($generation.arguments | get -o length) != null) {
      $args ++= [ --length ($generation.arguments.length) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "password" {
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    if (($generation.arguments | get -o length) != null) {
      $args ++= [ --length ($generation.arguments.length) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "password-crypt-3" {
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    if (($generation.arguments | get -o length) != null) {
      $args ++= [ --length ($generation.arguments.length) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "age-key" {
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "ssh-key" {
    $args ++= [ ($generation.arguments.name) ]
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "wireguard-key" {
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "key-split" {
    $args ++= [ ($generation.arguments.key) ]
    $args ++= [ ($generation.arguments.prefix) ]
    $args ++= [ ($generation.arguments.threshold) ]
    $args ++= [ ($generation.arguments.shares) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "key-combine" {
    $args ++= [ ($generation.arguments.shares | str join ',') ]
    $args ++= [ ($generation.arguments.key) ]
    $args ++= [ ($generation.arguments.threshold) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "nebula-ca" {
    $args ++= [ ($generation.arguments.name) ]
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    if (($generation.arguments | get -o days) != null) {
      $args ++= [ --days ($generation.arguments.days) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "nebula-cert" {
    $args ++= [ ($generation.arguments.ca_public) ]
    $args ++= [ ($generation.arguments.ca_private) ]
    $args ++= [ ($generation.arguments.name) ]
    $args ++= [ ($generation.arguments.ip) ]
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "cockroach-ca" {
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "cockroach-node-cert" {
    $args ++= [ ($generation.arguments.ca_public) ]
    $args ++= [ ($generation.arguments.ca_private) ]
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    $args ++= [ ($generation.arguments.hosts | str join ",") ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "cockroach-client-cert" {
    $args ++= [ ($generation.arguments.ca_public) ]
    $args ++= [ ($generation.arguments.ca_private) ]
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    $args ++= [ ($generation.arguments.user) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "tls-root" {
    $args ++= [ ($generation.arguments.common_name) ]
    $args ++= [ ($generation.arguments.organization) ]
    $args ++= [ ($generation.arguments.config) ]
    $args ++= [ ($generation.arguments.private) ]
    $args ++= [ ($generation.arguments.public) ]
    if (($generation.arguments | get -o pathlen) != null) {
      $args ++= [ --pathlen ($generation.arguments.pathlen) ]
    }
    if (($generation.arguments | get -o days) != null) {
      $args ++= [ --days ($generation.arguments.days) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "tls-intermediary" {
    $args ++= [ $generation.arguments.common_name ]
    $args ++= [ $generation.arguments.organization ]
    $args ++= [ $generation.arguments.config ]
    $args ++= [ $generation.arguments.request_config ]
    $args ++= [ $generation.arguments.private ]
    $args ++= [ $generation.arguments.request ]
    $args ++= [ $generation.arguments.ca_public ]
    $args ++= [ $generation.arguments.ca_private ]
    $args ++= [ $generation.arguments.serial ]
    $args ++= [ $generation.arguments.public ]
    if (($generation.arguments | get -o pathlen) != null) {
      $args ++= [ --pathlen ($generation.arguments.pathlen) ]
    }
    if (($generation.arguments | get -o days) != null) {
      $args ++= [ --days ($generation.arguments.days) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "tls-leaf" {
    $args ++= [ ($generation.arguments.common_name) ]
    $args ++= [ ($generation.arguments.organization) ]
    $args ++= [ ($generation.arguments.sans | str join ",") ]
    $args ++= [ ($generation.arguments.config) ]
    $args ++= [ ($generation.arguments.request_config) ]
    $args ++= [ ($generation.arguments.private) ]
    $args ++= [ ($generation.arguments.request) ]
    $args ++= [ ($generation.arguments.ca_public) ]
    $args ++= [ ($generation.arguments.ca_private) ]
    $args ++= [ ($generation.arguments.serial) ]
    $args ++= [ ($generation.arguments.public) ]
    if (($generation.arguments | get -o days) != null) {
      $args ++= [ --days ($generation.arguments.days) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "tls-rsa-root" {
    $args ++= [ ($generation.arguments.common_name) ]
    $args ++= [ ($generation.arguments.organization) ]
    $args ++= [ ($generation.arguments.config) ]
    $args ++= [ ($generation.arguments.private) ]
    $args ++= [ ($generation.arguments.public) ]
    if (($generation.arguments | get -o pathlen) != null) {
      $args ++= [ --pathlen ($generation.arguments.pathlen) ]
    }
    if (($generation.arguments | get -o days) != null) {
      $args ++= [ --days ($generation.arguments.days) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "tls-rsa-intermediary" {
    $args ++= [ $generation.arguments.common_name ]
    $args ++= [ $generation.arguments.organization ]
    $args ++= [ $generation.arguments.config ]
    $args ++= [ $generation.arguments.request_config ]
    $args ++= [ $generation.arguments.private ]
    $args ++= [ $generation.arguments.request ]
    $args ++= [ $generation.arguments.ca_public ]
    $args ++= [ $generation.arguments.ca_private ]
    $args ++= [ $generation.arguments.serial ]
    $args ++= [ $generation.arguments.public ]
    if (($generation.arguments | get -o pathlen) != null) {
      $args ++= [ --pathlen ($generation.arguments.pathlen) ]
    }
    if (($generation.arguments | get -o days) != null) {
      $args ++= [ --days ($generation.arguments.days) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "tls-rsa-leaf" {
    $args ++= [ ($generation.arguments.common_name) ]
    $args ++= [ ($generation.arguments.organization) ]
    $args ++= [ ($generation.arguments.sans | str join ",") ]
    $args ++= [ ($generation.arguments.config) ]
    $args ++= [ ($generation.arguments.request_config) ]
    $args ++= [ ($generation.arguments.private) ]
    $args ++= [ ($generation.arguments.request) ]
    $args ++= [ ($generation.arguments.ca_public) ]
    $args ++= [ ($generation.arguments.ca_private) ]
    $args ++= [ ($generation.arguments.serial) ]
    $args ++= [ ($generation.arguments.public) ]
    if (($generation.arguments | get -o days) != null) {
      $args ++= [ --days ($generation.arguments.days) ]
    }
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "tls-dhparam" {
    $args ++= [ ($generation.arguments.name) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "env" {
    $args ++= [ ($generation.arguments.name) ]
    $args ++= [ json ]
    let variables = $"($generation.arguments.name)-variables"
    $args ++= [ ($variables) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
      $generation.arguments.variables
        | rumor format write $variables json --renew --public
    } else {
      $generation.arguments.variables
        | rumor format write $variables json --public
    }
  } else if $generation.generator == "moustache" {
    $args ++= [ ($generation.arguments.name) ]
    $args ++= [ json ]
    let variables_and_template = (
      $"($generation.arguments.name)-variables-and-template")
    $args ++= [ ($variables_and_template) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
      {
        variables: $generation.arguments.variables
        template: $generation.arguments.template
      } | to json | rumor save $variables_and_template true
    } else {
      {
        variables: $generation.arguments.variables
        template: $generation.arguments.template
      } | to json | rumor save $variables_and_template false
    }
  } else if $generation.generator == "script" {
    if not $allow_script {
      (rumor log --error
        ("Running script generator not allowed."
          + " Please run with `--allow-script`"))
      exit 1
    }
    $args ++= [ ($generation.arguments.name) ]
    $args ++= [ ($generation.arguments.text) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
    }
  } else if $generation.generator == "sops" {
    $args ++= [ ($generation.arguments.age) ]
    $args ++= [ ($generation.arguments.public) ]
    $args ++= [ ($generation.arguments.private) ]
    $args ++= [ json ]
    let secrets = $"($generation.arguments.private)-secrets"
    $args ++= [ ($secrets) ]
    if (($generation.arguments | get -o renew) != null
      and $generation.arguments.renew) {
      $args ++= [ --renew ]
      $generation.arguments.secrets
        | rumor format write $secrets json --renew --public
    } else {
      $generation.arguments.secrets
        | rumor format write $secrets json --public
    }
  }

  rumor exec module $"generate ($generator)" ...($args)
}

def "rumor run export" [export: any]: nothing -> nothing {
  mut args = [ ]

  if ($export.exporter == "vault") {
    $args ++= [ ($export.arguments.path) ]
  } else if ($export.exporter == "vault-file") {
    $args ++= [ ($export.arguments.path) ]
    $args ++= [ ($export.arguments.file) ]
  } else if ($export.exporter == "copy") {
    $args ++= [ ($export.arguments.from) ]
    $args ++= [ ($export.arguments.to) ]
  }

  rumor exec module $"export ($export.exporter)" ...($args)
}

def "rumor create manifest" [
  script: string,
  schema: string,
  commandline: string,
  parsed_commandline: any,
  specification: string,
  specification_format: string,
  phase: string,
  error: any,
  with_output: bool,
  format: string
]: nothing -> string {
  let input =  {
    script_text: $script
    script_hash: ($script | hash sha256)
    schema_text: $schema,
    schema_hash: ($schema | hash sha256)
    specification_text: $specification
    specification_format: $specification_format
    specification_hash: ($specification | hash sha256)
    commandline: $commandline
    parsed_commandline: ($parsed_commandline | to json)
    commandline_hash: ($commandline | hash sha256)
  }
  let input = $input
    | insert hash ($input | rumor record hash)

  mut manifest: any = null
  if $with_output {
    let output = {
      files: (ls
        | where {
            not ($in.name
              | path basename
              | str starts-with rumor)
          }
        | sort-by name
        | each {
             {
               name: $in.name
               mode: (ls -la $in.name | get 0.mode)
               size: $in.size
               hash: (open --raw $in.name | hash sha256)
             }
          })
    }
    let output = $output
      | insert hash ($output | rumor record hash)

    $manifest = {
      version: $version
      phase: $phase
      error: $error
      time: (rumor now)
      input: $input
      output: $output
      format: $format
    }
  } else {
    $manifest = {
      version: $version
      phase: $phase
      error: $error
      time: (rumor now)
      input: $input
      format: $format
    }
  }

  $manifest
}

###############################################################################
# GENERATOR FUNCTIONS
###############################################################################

def "rumor tls root" [
  module: string,
  algorithm: string,
  options: string,
  common_name: string,
  organization: string,
  config: path,
  private: path,
  public: path,
  pathlen: int,
  days: int,
  renew: bool
]: nothing -> nothing {
  let basic_constraints = if $pathlen < 0 {
    $"critical,CA:true"
  } else {
    $"critical,CA:true,pathlen:($pathlen)"
  }

  $"
    [req]
    default_md = sha256
    distinguished_name = dn
    x509_extensions = ext
    prompt = no

    [dn]
    CN = ($common_name)
    O = ($organization)

    [ext]
    basicConstraints = ($basic_constraints)
    keyUsage = critical,keyCertSign,cRLSign
    subjectKeyIdentifier = hash
  " | str trim
    | rumor dedent 4
    | rumor save $config $renew --public

  (rumor exec tool $module
    openssl genpkey
    -algorithm $algorithm
    -pkeyopt $options
    -quiet)
    | rumor save $private $renew

  (rumor exec tool $module
    openssl req -x509
    -key $private
    -config $config
    -days ($days | into string))
    | rumor save $public $renew --public
}

def "rumor tls intermediary" [
  module: string,
  algorithm: string,
  options: string,
  common_name: string,
  organization: string,
  config: path,
  request_config: path,
  private: path,
  request: path,
  ca_public: path,
  ca_private: path,
  serial: path,
  public: path,
  pathlen: int,
  days: int,
  renew: bool
]: nothing -> nothing {
  let basic_constraints = if $pathlen < 0 {
    $"critical,CA:true"
  } else {
    $"critical,CA:true,pathlen:($pathlen)"
  }

  $"
    [req]
    default_md = sha256
    distinguished_name = dn
    x509_extensions = ext
    prompt = no

    [dn]
    CN = ($common_name)
    O = ($organization)

    [ext]
    keyUsage = critical,keyCertSign,cRLSign
    subjectKeyIdentifier = hash
  " | str trim
    | rumor dedent 4
    | rumor save $config $renew --public

  ((open --raw $request_config) + "\n" +
  ($"
    basicConstraints = ($basic_constraints)
    authorityKeyIdentifier = keyid,issuer
  " | str trim
    | rumor dedent 4))
    | rumor save $config $renew --public

  (rumor exec tool $module
    openssl genpkey
    -algorithm $algorithm
    -pkeyopt $options
    -quiet)
    | rumor save $private $renew

  (rumor exec tool $module
    openssl req -new
    -key $private
    -config $config
    -quiet)
    | rumor save $request $renew

  let serial_args = if ($serial | path exists) {
    cp $serial $"($serial)($tmp_suffix)"
    [ -CAserial $"($serial)($tmp_suffix)" ]
  } else {
    [ -CAcreateserial -CAserial $"($serial)($tmp_suffix)" ]
  }

  (rumor exec tool $module
    openssl x509 -req
    -in $request
    -CA $ca_public
    -CAkey $ca_private
    ...($serial_args)
    -extfile $config
    -extensions ext
    -days ($days | into string))
    | rumor save $public $renew --public

  let serial_content = open --raw $"($serial)($tmp_suffix)"
  rm -f $"($serial)($tmp_suffix)"
  $serial_content | rumor save $serial $renew
}

def "rumor tls leaf" [
  module: string,
  algorithm: string,
  options: string,
  common_name: string,
  organization: string,
  sans: string,
  config: path,
  request_config: path,
  private: path,
  request: path,
  ca_public: path,
  ca_private: path,
  serial: path,
  public: path,
  days: int,
  renew: bool
]: nothing -> nothing {
  let key_usage = if ($algorithm == "RSA") {
    "critical,digitalSignature,keyEncipherment"
  } else {
    "critical,digitalSignature"
  }

  let sans = $sans | split row ","

  let ip_sans = $sans
    | where { $in | rumor is ip }
    | enumerate
    | each { $"IP.($in.index + 1) = ($in.item)" }
    | str join "\n"

  let dns_sans = $sans
    | where { not ($in | rumor is ip) }
    | enumerate
    | each { $"DNS.($in.index + 1) = ($in.item)" }
    | str join "\n"

  $"
    [req]
    default_md = sha256
    distinguished_name = dn
    req_extensions = ext
    prompt = no

    [dn]
    CN = ($common_name)
    O = ($organization)

    [sans]
    ($dns_sans)
    ($ip_sans)

    [ext]
    keyUsage = ($key_usage)
    extendedKeyUsage = serverAuth,clientAuth
    subjectAltName = @sans
    subjectKeyIdentifier = hash
  " | str trim
    | rumor dedent 4
    | rumor save $request_config $renew --public

  ((open --raw $request_config) + "\n" +
  ($"
    basicConstraints = critical,CA:false
    authorityKeyIdentifier = keyid,issuer
  " | str trim
    | rumor dedent 4))
    | rumor save $config $renew --public

  (rumor exec tool $module
    openssl genpkey
    -algorithm $algorithm
    -pkeyopt $options
    -quiet)
    | rumor save $private $renew

  (rumor exec tool $module
    openssl req -new
    -key $private
    -config $request_config
    -quiet)
    | rumor save $request $renew

  let serial_args = if ($serial | path exists) {
    cp $serial $"($serial)($tmp_suffix)"
    [ -CAserial $"($serial)($tmp_suffix)" ]
  } else {
    [ -CAcreateserial -CAserial $"($serial)($tmp_suffix)" ]
  }

  (rumor exec tool $module
    openssl x509 -req
    -in $request
    -CA $ca_public
    -CAkey $ca_private
    ...($serial_args)
    -extfile $config
    -extensions ext
    -days ($days | into string))
    | rumor save $public $renew --public

  let serial_content = open --raw $"($serial)($tmp_suffix)"
  rm -f $"($serial)($tmp_suffix)"
  $serial_content | rumor save $serial $renew
}

def "rumor secure random alnum" [
  module: string,
  length: int
]: nothing -> string {
  mut out = ""
  while ($out | str length) < $length {
    let need = $length - ($out | str length)
    let batch = (rumor exec tool $module
      openssl rand -base64 ([ ($need * 2) 32 ] | math max ))
      | str replace -a -r '[^A-Za-z0-9]' ''
      | split row ""
      | skip 1
      | take $need
      | str join ""
    $out += $batch
  }
  $out
}

def "rumor secure random digits" [
  module: string,
  length: int
]: nothing -> string {
  mut out = ""
  while ($out | str length) < $length {
    let need = $length - ($out | str length)
    let batch = (rumor exec tool $module
      openssl rand -base64 ([ ($need * 2) 32 ] | math max ))
      | str replace -a -r '[^0-9]' ''
      | split row ""
      | skip 1
      | take $need
      | str join ""
    $out += $batch
  }
  $out
}

###############################################################################
# FILESYSTEM FUNCTIONS
###############################################################################

def "rumor purge workdir" []: nothing -> nothing {
  ls -a | each { rm -rf $in.name }
  return
}

def "rumor mktemp" [
  path: string,
  --directory,
  --suffix = ""
]: nothing -> path {
  if $directory {
    let result = mktemp -t --suffix $suffix -d $"rumor-($path)-XXXX"
    chmod 700 $result
    $result
  } else {
    let result = mktemp -t --suffix $suffix $"rumor-($path)-XXXX"
    chmod 600 $result
    return $result
  }
}

def "rumor save" [
  path: path,
  renew: bool,
  --public
]: string -> nothing {
  $in | save -f $"($path)($tmp_suffix)"
  if $renew {
    mv -f $"($path)($tmp_suffix)" $path
  } else {
    try { mv -n $"($path)($tmp_suffix)" $path }
  }
  rm -f $"($path)($tmp_suffix)"
  if ($public) {
    chmod 644 $path
  } else {
    chmod 600 $path
  }
}

###############################################################################
# LOGGING FUNCTIONS
###############################################################################

def --wrapped "rumor exec module" [
  module: string,
  ...args: string
]: nothing -> nothing {
  let verbose = $env.__RUMOR_VERBOSE? | is-not-empty

  let args = $args
    | each { into string }
    | each {
        if (($in | str contains " ")
          or $in == "false"
          or $in == "true"
          or $in == "null") {
          $"`($in)`"
        } else {
          $in
        }
      }
    | str join " "

  let result = (try { nu -c $"exec nu ($main) ($module) ($args)" }
    | complete)

  if $result.exit_code != 0 {
    let message = ($"module '($module)' with args '($args)'"
      + $" exited with '($result.exit_code)'")
    $result.stderr |
      (rumor log
        --module $module
        --error $message)
    error make { msg: $message }
  }

  if $verbose {
    $result.stderr |
      (rumor log
        --module $module
        --message $"called '($module)' with args '($args)'")
  }
}

def --wrapped "rumor exec tool" [
  module: string,
  tool: string,
  ...args: string
]: [
  string -> string,
  string -> nothing,
  nothing -> string,
  nothing -> nothing
] {
  let very_verbose = $env.__RUMOR_VERY_VERBOSE? | is-not-empty

  let stdin = $in
  let args = $args
    | each { into string }
    | each {
        if (($in | str contains " ")
          or $in == "false"
          or $in == "true"
          or $in == "null") {
          $"`($in)`"
        } else {
          $in
        }
      }
    | str join " "

  let result = if ($stdin | is-not-empty) and ($stdin | str trim | is-not-empty) {
    try { $stdin | nu --stdin -c $"exec ($tool) ($args)"  }
      | complete
  } else {
    try { nu -c $"exec ($tool) ($args)" }
      | complete
  }

  if $result.exit_code != 0 {
    let message = ($"tool '($tool)' with args '($args)'"
      + $" exited with '($result.exit_code)'")
    $result.stderr |
      (rumor log
        --module $module
        --error $message)
    error make { msg: $message }
  }

  if $very_verbose {
    $result.stderr |
      (rumor log
        --module $module
        --message $"called '($tool)' with args '($args)'")
  }

  return $result.stdout
}

def "rumor log" [
  --error: string = "",
  --message: string = "",
  --module: string = "rumor",
  --debug: string = ""
]: nothing -> nothing, string -> nothing {
  let time = rumor now

  mut stdin = $in
  let has_stdin = $stdin | str trim | is-not-empty
  $stdin = if $has_stdin { $stdin | rumor indent 2 } else { "" }

  if ($debug | str trim | is-not-empty) {
    if $has_stdin {
      print --stderr $"[($time)][DEBUG]: ($debug)\n($stdin)"
    } else {
      print --stderr $"[($time)][DEBUG]: ($debug)"
    }
  } else if ($error | str trim | is-not-empty) {
    if $has_stdin {
      print --stderr $"[($time)][($module)]: ($error)\n($stdin)"
    } else {
      print --stderr $"[($time)][($module)]: ($error)"
    }
  } else if ($message | str trim | is-not-empty) {
    if $has_stdin {
      print $"[($time)][($module)]: ($message)\n($stdin)"
    } else {
      print $"[($time)][($module)]: ($message)"
    }
  }
}

###############################################################################
# FORMAT FUNCTIONS
###############################################################################

def "rumor format read" [path: path, format?: string]: nothing -> any {
  mut format = $format
  if ($format | is-not-empty) and ($format | str trim | is-not-empty) {
    if not (rumor format valid $format) {
      return null
    }
  } else {
    $format = (rumor format detect $path)
    if ($format | is-empty) {
      return null
    }
  }

  open --raw $path | rumor format deserialize $format
}

def "rumor format write" [
  path: path,
  format: string,
  --renew,
  --public
]: any -> nothing {
  if not (rumor format valid $format) {
    return
  }

  let serialized = $in | rumor format serialize $format
  if $public {
    $serialized | rumor save $path $renew --public
    $format | rumor save $"($path)($format_suffix)" $renew --public
  } else {
    $serialized | rumor save $path $renew
    $format | rumor save $"($path)($format_suffix)" $renew
  }
}

def "rumor format detect" [path: path]: nothing -> string {
  let format = if ($"($path)($format_suffix)" | path exists) {
    open --raw $"($path)($format_suffix)"
  } else {
    $path | path parse | get extension
  }

  if not (rumor format valid $format) {
    return ""
  }
  return $format
}

def "rumor format valid" [format: string]: nothing -> bool {
  if ($format in [ "json" "yaml" "yml" "toml" ]) {
    return true
  }

  return false
}

def "rumor format deserialize" [format: string]: string -> any {
  if $format == "json" {
    $in | from json
  } else if $format == "yaml" {
    $in | from yaml
  } else if $format == "yml" {
    $in | from yaml
  } else if $format == "toml" {
    $in | from toml
  } else {
    return null
  }
}

def "rumor format serialize" [format: string]: any -> string {
  if $format == "json" {
    $in | to json
  } else if $format == "yaml" {
    $in | to yaml
  } else if $format == "yml" {
    $in | to yaml
  } else if $format == "toml" {
    $in | to toml
  } else {
    return ""
  }
}

###############################################################################
# MISCELLANEOUS FUNCTIONS
###############################################################################

def "rumor now" []: nothing -> string {
  date now | format date $timestamp_format
}

def "rumor vault now" []: nothing -> string {
  date now | format date $vault_timestamp_format
}

def "rumor decode if bytes" []: any -> string {
  if ($in | describe) == "string" {
    $in
  } else {
    $in | decode
  }
}

def "rumor escape env string" []: string -> string {
  $in
    | str replace -a "\\" "\\\\"
    | str replace -a "\"" "\\\""
    | str replace -a "\n" "\\n"
    | str replace -a "\r" "\\r"
    | str trim
}

def "rumor escape arg string" []: string -> string {
  $in
    | str replace -a "\\" "\\\\"
    | str replace -a "\"" "\\\""
    | str replace -a "\n" "\\n"
    | str replace -a "\r" "\\r"
    | str trim
}

def "rumor record hash" []: record -> string {
  transpose key value
    | sort-by key
    | to json -r
    | hash sha256
}

def "rumor indent" [amount: int]: string -> string {
  let indent = 1..($amount)
    | each { " " }
    | str join ""

  $in
    | split row "\n"
    | each { $indent + $in }
    | str join "\n"
}

def "rumor dedent" [amount: int]: string -> string {
  let indent = 1..($amount)
    | each { " " }
    | str join ""

  $in
    | split row "\n"
    | each {
        if ($in | str starts-with $indent) {
          ($in | parse --regex $"($indent)\(.*\)").0.capture0
        } else {
          $in | str trim --left
        }
      }
    | str join "\n"
}

def "rumor is ip" []: string -> bool {
  ($in | parse --regex $ip_regex).0?.capture0? == $in
}
