#!/bin/bash

cp ../target/release/sherpax .
cp ../../polkadot/target/release/polkadot .

bash ./regenerateConfig-rococo-local.sh


docker build . -f ParaDockerfile -t comingweb3/sherpax:v0.9.9
docker build . -f RelayDockerfile -t comingweb3/polkadot:v0.9.9

#rm ./polkadot
#rm ./dev-parachain

