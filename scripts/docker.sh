#! /bin/bash

docker build -t docker.pkg.github.com/growbe2/growbe-mainboard/dev:latest -f ./docker/baseRust.Dockerfile $(pwd)