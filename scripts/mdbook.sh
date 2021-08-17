#!/bin/bash

cargo install mdbook

pushd docs
mdbook build
mdbook test
popd
