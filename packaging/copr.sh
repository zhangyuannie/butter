#!/bin/bash
#
# copr custom source

set -euo pipefail

# Build dependencies: bash jq curl rpmdevtools
# Result directory: out
out="${PWD}/out"

raw_src_url="https://raw.githubusercontent.com/zhangyuannie/butter/main"

commit_json=$(curl -sS -H "Accept: application/vnd.github.v3+json" \
  https://api.github.com/repos/zhangyuannie/butter/commits/main)

sha=$(printf '%s\n' "${commit_json}" | jq -r '.sha')

date=$(printf '%s\n' "${commit_json}" | jq -r '.commit.committer.date')
formatted_date=$(date --date "${date}" +%Y%m%d)

version=$(curl -sS "${raw_src_url}/meson.build" |
  grep -A 3 "\<project(" | grep "\<version\>" |
  sed "s|.*version: '\(.*\)'.*|\1|")

mkdir -p "${out}"

curl -sS "${raw_src_url}/packaging/butter.spec.in" |
  sed -e "s|@COMMITDATE@|${formatted_date}|" \
      -e "s|@COMMIT@|${sha}|" \
      -e "s|@VERSION@|${version}|" \
  > "${out}/butter.spec"

spectool -g -C "${out}" "${out}/butter.spec"
