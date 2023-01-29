self: {
  config,
  lib,
  ...
}: {
  options.nh = with lib; {
    enable = mkOption {
      type = types.bool;
      default = true;
      description = "Enables NH and any checks needed";
    };

    package = mkOption {
      type = types.package;
      default = self.packages.${config.nixpkgs.system}.default;
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
        description = "How often cleaning is performed. Passed to systemd.time";
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
        script = "exec ${config.nh.package}/bin/nh clean";
        startAt = config.nh.clean.dates;
      };

      timers.nh-clean = {
        timerConfig = {
          Persistent = true;
        };
      };
    };
  };
}
