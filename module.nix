self: {
  config,
  lib,
  pkgs,
  ...
}: {
  options.nh = with lib; {
    enable = mkOption {
      type = types.bool;
      default = true;
      description = "Adds nh to your package list";
    };

    package = mkOption {
      type = types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      description = "Which NH package to use";
    };

    flake = mkOption {
      type = with types; nullOr path;
      default = null;
      description = "The path that will be used for the `FLAKE` environment variable";
    };

    clean = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enables periodic cleaning";
      };

      dates = mkOption {
        type = types.str;
        default = "weekly";
        description = "How often cleaning is triggered. Passed to systemd.time";
      };

      extraArgs = mkOption {
        type = types.str;
        default = "";
        example = "--keep 5 --keep-since 3d";
        description = "Flags passed to nh clean all";
      };
    };
  };

  config = lib.mkIf config.nh.enable {
    assertions = [
      {
        assertion = config.nh.clean.enable -> config.nh.enable;
        message = "nh.clean.enable requires nh.enable";
      }

      {
        assertion = (config.nh.flake != null) -> !(lib.hasSuffix ".nix" config.nh.flake);
        message = "nh.flake must be a directory";
      }
    ];

    environment = {
      systemPackages = [config.nh.package];
      variables = lib.optionalAttrs (config.nh.flake != null) {
        FLAKE = config.nh.flake;
      };
    };

    systemd = lib.mkIf config.nh.clean.enable {
      services.nh-clean = {
        description = "NH cleaner";
        script = "exec ${config.nh.package}/bin/nh clean all ${config.nh.clean.extraArgs}";
        startAt = config.nh.clean.dates;
        path = [config.nix.package];
      };

      timers.nh-clean = {
        timerConfig = {
          Persistent = true;
        };
      };
    };
  };
}
