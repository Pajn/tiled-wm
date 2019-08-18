#!/bin/sh

cd $(dirname $0)
cargo build $@

cbindgen --config cbindgen.toml --crate compository --output compository.h