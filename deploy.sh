#!/usr/bin/env bash

set -e

# docker build triggers a production install with cargo

# x86_64
########
#docker build -t peregin/velocorner.rates .
#docker push peregin/velocorner.rates:latest

# aarch64
#########
docker buildx build --platform linux/amd64,linux/arm64 -t peregin/velocorner.rates:latest --push .

# test the image if needed
#docker run --rm -it -p 9012:9012 peregin/velocorner.rates

echo "Successfully deployed..."

