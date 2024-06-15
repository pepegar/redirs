{ pkgs, lib, config, inputs, ... }:

{
  packages = [ pkgs.rust-analyzer ];
  
  languages.nix.enable = true;
  languages.rust.enable = true;
}
