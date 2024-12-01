{
  rustPlatform,
  libclang,
  lib,
  buildInputsArr,
  nativeBuildInputsArr,
  ...
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  lockFile = ../Cargo.lock;
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = ../.;

  cargoLock = {
    outputHashes = {
      "bevy_iced-0.5.0" = "sha256-P2TAARTh8oCDxWOipUzjr2nMOvLuODjmmQ581iSalQs=";
      "glyphon-0.5.0" = "sha256-wqpU1ioIWHV8gL6aqEO/H2SoQ6xccT/ZR9QMHygZaNw=";
      "iced_core-0.12.1" = "sha256-UxUPGsB3+vKZjuCbPAwneTgwSH4vicOuU97ppDpAGaY=";
    };
    inherit lockFile;
  };

  buildInputs = buildInputsArr;
  nativeBuildInputs = nativeBuildInputsArr;
  LD_LIBRARY_PATH = lib.makeLibraryPath nativeBuildInputs;
  LIBCLANG_PATH = "${libclang.lib}/lib";
  copyLibs = true;

  meta = with lib; {
    description = "A simple tank game";
    homepage = "https://github.com/Xetibo/tankrs";
    changelog = "https://github.com/Xetibo/tankrs/releases/tag/${version}";
    license = licenses.gpl3;
    maintainers = with maintainers; [ DashieTM ];
    mainProgram = "tankrs";
  };
}
