#!/bin/sh

$HOME/.cargo/bin/bindgen odpi/include/dpi.h -o src/binding/binding.rs \
  --distrust-clang-mangling \
  --whitelist-type "^dpi.*" \
  --whitelist-function "^dpi.*" \
  --whitelist-var "^DPI_.*" \
  --bitfield-enum dpiExecMode \
  --bitfield-enum dpiFetchMode \
  --bitfield-enum dpiOpCode \
  --bitfield-enum dpiSubscrQOS \
  --no-prepend-enum-name \
  --with-derive-default \
  --rust-target 1.47 \
  -- -Iodpi/include

$HOME/.cargo/bin/bindgen odpi/src/dpiImpl.h -o src/binding/binding_impl.rs \
  --whitelist-var "DPI_MAX_BASIC_BUFFER_SIZE" \
  --whitelist-var "DPI_NUMBER_AS_TEXT_CHARS" \
  --whitelist-var "DPI_OCI_HTYPE_SVCCTX" \
  --whitelist-var "DPI_OCI_HTYPE_SERVER" \
  --whitelist-var "DPI_OCI_HTYPE_SESSION" \
  --rust-target 1.47 \
  -- -Iodpi/include
