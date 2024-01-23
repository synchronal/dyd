{ lib, fetchFromGitHub, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "dyd";
  version = "1.8.0";

  src = fetchFromGitHub {
    owner = "synchronal";
    repo = "dyd";
    rev = "v1.8.0";
    hash = "sha256-DmdrOacLOxb8v5niYmSCmxvU3bcLzRlZa+JROn4HDHE=";
  };

  cargoHash = "sha256-45gqEanUjzXNJz/muWps/HUJqKLx8/YCh18L4gIJhSk=";

  meta = with lib; {
    description = "Daily diff";
    homepage = "https://github.com/synchronal/dyd";
    license = licenses.mit;
    maintainers = ["synchronal.dev"];
  };
}
