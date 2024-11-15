#!/usr/bin/env bash

set -e

# docker build triggers a production install with cargo
# build for ARM only, otherwise CI takes too much time
#########
# Check if multi-arch-builder exists
if docker buildx inspect multi-arch-builder &> /dev/null; then
    echo "buildx exists..."
else
    echo "buildx does not exist... creating it..."
    docker buildx create --name multi-arch-builder
fi
docker buildx use multi-arch-builder
docker buildx build --platform linux/arm64 -t peregin/velocorner.rates:latest --push .

echo "Successfully deployed..."

