{ lib, rustPlatform }:

let
  cargoToml = builtins.readFile ./Cargo.toml;
  versionMatches = builtins.match ".*\nversion = \"([0-9.]+)\".*" cargoToml;
  version = builtins.elemAt versionMatches 0;
in

rustPlatform.buildRustPackage {
  pname = "dyd";
  inherit version;

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with lib; {
    description = "Daily diff";
    homepage = "https://github.com/synchronal/dyd";
    license = licenses.mit;
    maintainers = [ "synchronal.dev" ];
  };
}
