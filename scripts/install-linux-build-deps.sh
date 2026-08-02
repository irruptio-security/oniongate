#!/usr/bin/env bash
# Install everything needed to compile and bundle OnionGate on Debian/Ubuntu.
#
# Kept as one script so CI, release, and packaging workflows cannot drift apart.
set -euo pipefail

sudo apt-get update

sudo apt-get install -y \
  build-essential \
  curl \
  file \
  wget \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libxdo-dev \
  patchelf \
  desktop-file-utils

# AppImage tooling is itself distributed as an AppImage and needs FUSE 2, which
# Ubuntu stopped shipping by default in 22.04.
sudo apt-get install -y libfuse2 || sudo apt-get install -y libfuse2t64

# The Tor sidecar links against libevent. linuxdeploy refuses to build an
# AppImage when it cannot resolve a bundled binary's dependencies, so the
# library has to exist on the builder even though only Tor consumes it.
sudo apt-get install -y libevent-2.1-7
