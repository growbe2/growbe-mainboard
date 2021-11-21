#! /bin/bash


TARGET=$1
USER=wq

sudo mkdir -p /opt/growbe/growbe-mainboard && sudo chown -R $USER /opt/growbe && \
sudo cp ./scripts/growbe-mainboard.service /usr/lib/systemd/system

# doit avoir fait le build release pour la plateform

# doit prendre le bonne execuable par plateforme
cp ./target/$TARGET/growbe-mainboard ./virtual-comboard.json ./mainboard_config.json /opt/growbe/growbe-mainboard




