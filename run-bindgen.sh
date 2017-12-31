#!/bin/sh

$HOME/.cargo/bin/bindgen odpi/include/dpi.h -o src/binding.rs \
  --distrust-clang-mangling \
  --whitelist-type "^dpi.*" \
  --whitelist-function "^dpi.*" \
  --whitelist-var "^DPI_.*" \
  --bitfield-enum dpiExecMode \
  --bitfield-enum dpiFetchMode \
  --bitfield-enum dpiOpCode \
  --bitfield-enum dpiSubscrQOS \
  --no-prepend-enum-name \
  --rust-target 1.19 \
  -- -Iodpi/include
