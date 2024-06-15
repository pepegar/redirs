{ pkgs, lib, config, inputs, ... }:

{
  packages = [
    pkgs.rust-analyzer
    pkgs.redis
  ];
  
  languages.nix.enable = true;
  languages.rust.enable = true;
}
