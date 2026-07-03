{
  lib,
  stdenv,
  craneLib,
  pkg-config,
  opus,
  alsa-lib,
  src,
  pname,
  version,
  description,
  homepage,
}:

let
  commonArgs = {
    inherit pname version src;

    cargoExtraArgs = "--locked --features voice-playback";

    # audiopus_sys and ALSA use pkg-config to find system libraries.
    # Providing Opus here avoids falling back to bundled Opus CMake builds,
    # which are sensitive to the host CMake version.
    nativeBuildInputs = [ pkg-config ];

    # Networking uses rustls + webpki-roots, so we do not need openssl or a
    # system CA bundle here. Darwin stdenv provides the SDK by default, so avoid
    # legacy darwin.apple_sdk framework stubs.
    buildInputs = [
      opus
    ] ++ lib.optionals stdenv.isLinux [
      alsa-lib
    ];

    # The unit tests in this repo do not require network or a TTY, but disable
    # them by default to keep `nix build` fast and reproducible. Run `cargo test`
    # inside `nix develop` for the full test suite.
    doCheck = false;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (commonArgs // {
  inherit cargoArtifacts;

  meta = {
    inherit description homepage;
    license = lib.licenses.gpl3Only;
    mainProgram = "concord";
    platforms = lib.platforms.unix;
  };
})
