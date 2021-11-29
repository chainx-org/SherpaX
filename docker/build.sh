#!/bin/bash

cp ../target/release/sherpax .
cp ../../polkadot/target/release/polkadot .

bash ./regenerateConfig-rococo-local.sh


docker build . -f ParaDockerfile -t comingweb3/sherpax-dev:v0.9.11
docker build . -f RelayDockerfile -t comingweb3/polkadot:v0.9.11

#rm ./polkadot
#rm ./dev-parachain

