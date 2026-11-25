#!/usr/bin/env bash
# Copyright (c) 2026 Faris Alfarhan
# SPDX-License-Identifier: GPL-3.0-only

# Optional local toolchain override. CI and any machine with a working Node 24
# on PATH can ignore this file.
local_node="$HOME/.local/node-v26.13.0/bin"
if [ -d "$local_node" ]; then
  export PATH="$local_node:$PATH"
fi
