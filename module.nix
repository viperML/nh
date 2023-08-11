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
      default = self.packages.${pkgs.stdenv.hostPlatform}.default;
      description = "Which NH package to use";
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
    ];

    environment.systemPackages = [config.nh.package];

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
