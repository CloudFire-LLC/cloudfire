#!/usr/bin/env bash

set -euox pipefail

docker compose exec --env RUST_LOG=info -it client /bin/sh -c 'iperf3 \
  --reverse \
  --udp \
  --bandwidth 50M \
  --client 172.20.0.110 \
  --json' >>"${TEST_NAME}.json"

assert_process_state "firezone-gateway" "S"
assert_process_state "firezone-linux-client" "S"
