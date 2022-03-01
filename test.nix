{...}: {
  networking.wireguard.enable = true;
  networking.wireguard.interfaces.wg0 = {
    privateKeyFile = "/run/secrets/wireguard.wg0.private";
  };
}
