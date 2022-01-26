#!/bin/sh

if [ "$(id -u)" -eq 0 ]; then
  /usr/libexec/butter $*
else
  if command -v pkexec > /dev/null; then
    pkexec /usr/libexec/butter $*
  else
    echo "Butter must be run as root"
    exit 1
  fi
fi
