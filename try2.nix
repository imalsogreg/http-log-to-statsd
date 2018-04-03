{ pkgs ? import <nixpkgs> {}, makeWrapper, rustup }:
with pkgs.rustPlatform;

buildRustPackage rec {
  name = "http-log-to-statsd-${version}";
  version = "0.1.0";
  src = pkgs.fetchFromGitHub {
    owner = "imalsogreg";
    repo = "http-log-to-statsd";
    rev = "314ca4aea52b1ed89d7b9e7f0c4e22e86f468530";
    sha256 = "14zk7xcl99m3jjj16sx846i6nrvrdp8cgl054vl7s9vpkky5x2w3";
  };
  cargoSha256 = "13sb3ybibmy5p0ramy9m3zcz2kljybyvw7npal6y2h434565cphg";
  meta = with pkgs.stdenv.lib; {
    description = "A utility for forwarding http logs to statsd";
    homepage = https://github.com/caldwell/http-log-to-statsd;
    license = [];
    maintainers = [];
    platforms = platforms.all;
  };
  buildInputs = [ makeWrapper rustup ];
}
