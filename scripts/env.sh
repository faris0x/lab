#!/usr/bin/env bash
# Local toolchain for this machine: the system Node is broken because its
# libsimdjson soname does not match the installed library.
export PATH="$HOME/.local/node-v26.13.0/bin:$PATH"
