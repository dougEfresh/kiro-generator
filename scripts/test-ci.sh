#!/bin/bash

if [ -z "$CI" ]; then
  echo no running in CI >/dev/stderr
  exit 1
fi

mkdir -p .kiro "$HOME/.kiro"
cp -a ./data/kiro/generators .kiro
cp -a ./data/kiro/global/* "$HOME/.kiro"

KG=./target/debug/kg
cargo build

$KG help
$KG --help
$KG validate
$KG v
$KG v --debug
$KG v --trace aws-test --debug
$KG v --local
$KG v --global
$KG generate
$KG g
$KG diff
$KG schema manifest >/dev/null
$KG schema agent >/dev/null
