#!/bin/bash

if [[ "${ENABLE_MASQUERADE}" == "1" ]]; then
  IFACE="utun"
  iptables -A FORWARD -i $IFACE -j ACCEPT
  iptables -A FORWARD -o $IFACE -j ACCEPT
  iptables -t nat -A POSTROUTING -o eth+ -j MASQUERADE
  ip6tables -A FORWARD -i $IFACE -j ACCEPT
  ip6tables -A FORWARD -o $IFACE -j ACCEPT
  ip6tables -t nat -A POSTROUTING -o eth+ -j MASQUERADE
fi

if [[ "${LISTEN_ADDRESS_DISCOVERY_METHOD}" == "gce_metadata" ]]; then
  export PUBLIC_IP4_ADDR=$(curl "http://metadata.google.internal/computeMetadata/v1/instance/network-interfaces/0/access-configs/0/external-ip" -H "Metadata-Flavor: Google" -s)
  export LISTEN_IP4_ADDR=$PUBLIC_IP4_ADDR
  echo "Discovered PUBLIC_IP4_ADDR: ${PUBLIC_IP4_ADDR}"
  echo "Discovered LISTEN_IP4_ADDR: ${LISTEN_IP4_ADDR}"
fi

# if first arg looks like a flag, assume we want to run postgres server
if [ "${1:0:1}" = '-' ]; then
  set -- ${PACKAGE_NAME} "$@"
fi

exec "$@"
