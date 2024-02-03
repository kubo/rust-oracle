#!/bin/sh

$HOME/.cargo/bin/bindgen odpi/include/dpi.h -o src/binding/binding.rs \
  --distrust-clang-mangling \
  --allowlist-type "^dpi.*" \
  --allowlist-function "^dpi.*" \
  --allowlist-var "^DPI_.*" \
  --bitfield-enum dpiExecMode \
  --bitfield-enum dpiFetchMode \
  --bitfield-enum dpiOpCode \
  --bitfield-enum dpiSubscrQOS \
  --no-prepend-enum-name \
  --with-derive-default \
  --rust-target 1.59 \
  -- -Iodpi/include

$HOME/.cargo/bin/bindgen odpi/src/dpiImpl.h -o src/binding/binding_impl.rs \
  --allowlist-var "DPI_MAX_BASIC_BUFFER_SIZE" \
  --allowlist-var "DPI_NUMBER_AS_TEXT_CHARS" \
  --allowlist-var "DPI_OCI_HTYPE_SVCCTX" \
  --allowlist-var "DPI_OCI_HTYPE_SERVER" \
  --allowlist-var "DPI_OCI_HTYPE_SESSION" \
  --rust-target 1.59 \
  -- -Iodpi/include
