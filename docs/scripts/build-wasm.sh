#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
DOCS_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
OUT_DIR="$DOCS_DIR/.vuepress/public/wasm/pkg"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

cd "$ROOT_DIR"
wasm-pack build docs/wasm \
  --target web \
  --out-dir "$OUT_DIR" \
  --out-name rquickjs_playground \
  --no-pack

python - <<'PY' "$OUT_DIR"
from pathlib import Path
import sys

out_dir = Path(sys.argv[1])
js_path = out_dir / "rquickjs_playground.js"
host_path = out_dir / "rquickjs_host.js"
source = js_path.read_text()

host_import = "import { setRquickjsMemory } from './rquickjs_host.js';\n"
if "setRquickjsMemory" not in source:
    if source.startswith("/* @ts-self-types="):
        first_newline = source.find("\n")
        source = source[: first_newline + 1] + host_import + source[first_newline + 1 :]
    else:
        source = host_import + source

if "setRquickjsMemory(wasm.memory);" not in source:
    source = source.replace(
        "    wasm = instance.exports;\n",
        "    wasm = instance.exports;\n    setRquickjsMemory(wasm.memory);\n",
        1,
    )

js_path.write_text(source)
host_path.write_text(
    """const decoder = new TextDecoder('utf-8');
const encoder = new TextEncoder();

let memory = null;
let heapTop = 1024;
const allocations = new Map();

function requireMemory() {
  if (!memory) {
    throw new Error('rquickjs host memory is not initialized');
  }
  return memory;
}

function bytes() {
  return new Uint8Array(requireMemory().buffer);
}

function align(value) {
  return (value + 7) & ~7;
}

function alloc(size) {
  const alignedSize = align(Math.max(size, 1));
  const ptr = heapTop;
  heapTop += alignedSize;
  allocations.set(ptr, alignedSize);
  return ptr;
}

function readCString(ptr) {
  const view = bytes();
  let end = ptr;
  while (view[end] !== 0) end += 1;
  return decoder.decode(view.subarray(ptr, end));
}

function writeCString(ptr, text, maxBytes = Infinity) {
  const view = bytes();
  const encoded = encoder.encode(text);
  const limit = Math.max(0, Math.min(encoded.length, maxBytes - 1));
  view.set(encoded.subarray(0, limit), ptr);
  view[ptr + limit] = 0;
  return limit;
}

function writeI32(ptr, value) {
  if (!ptr) return;
  new DataView(requireMemory().buffer).setInt32(ptr, value, true);
}

function writePtr(ptr, value) {
  if (!ptr) return;
  new DataView(requireMemory().buffer).setUint32(ptr, value, true);
}

function fillTimeStruct(tmPtr, date) {
  const view = new DataView(requireMemory().buffer);
  view.setInt32(tmPtr + 0, date.getUTCSeconds(), true);
  view.setInt32(tmPtr + 4, date.getUTCMinutes(), true);
  view.setInt32(tmPtr + 8, date.getUTCHours(), true);
  view.setInt32(tmPtr + 12, date.getUTCDate(), true);
  view.setInt32(tmPtr + 16, date.getUTCMonth(), true);
  view.setInt32(tmPtr + 20, date.getUTCFullYear() - 1900, true);
  view.setInt32(tmPtr + 24, date.getUTCDay(), true);
  view.setInt32(tmPtr + 28, 0, true);
  view.setInt32(tmPtr + 32, 0, true);
  return tmPtr;
}

export function setRquickjsMemory(nextMemory) {
  memory = nextMemory;
  if (heapTop >= bytes().byteLength) {
    heapTop = 1024;
  }
}

export function installRquickjsEnv(env = {}) {
  return Object.assign(env, {
    malloc: alloc,
    calloc(count, size) {
      const total = count * size;
      const ptr = alloc(total);
      bytes().fill(0, ptr, ptr + total);
      return ptr;
    },
    realloc(ptr, size) {
      if (!ptr) return alloc(size);
      const next = alloc(size);
      const view = bytes();
      const previous = allocations.get(ptr) || 0;
      view.copyWithin(next, ptr, ptr + Math.min(previous, size));
      allocations.delete(ptr);
      return next;
    },
    free(ptr) {
      allocations.delete(ptr);
    },
    strlen(ptr) {
      return readCString(ptr).length;
    },
    memchr(ptr, ch, count) {
      const view = bytes();
      for (let i = 0; i < count; i += 1) {
        if (view[ptr + i] === ch) return ptr + i;
      }
      return 0;
    },
    strchr(ptr, ch) {
      const view = bytes();
      let offset = ptr;
      while (view[offset] !== 0) {
        if (view[offset] === ch) return offset;
        offset += 1;
      }
      return 0;
    },
    strrchr(ptr, ch) {
      const view = bytes();
      let match = 0;
      let offset = ptr;
      while (view[offset] !== 0) {
        if (view[offset] === ch) match = offset;
        offset += 1;
      }
      return match;
    },
    strcmp(a, b) {
      return readCString(a).localeCompare(readCString(b));
    },
    strncmp(a, b, count) {
      return readCString(a).slice(0, count).localeCompare(readCString(b).slice(0, count));
    },
    strtod(ptr, endPtr) {
      const text = readCString(ptr);
      const match = text.match(/^[\\t\\n\\r ]*([+-]?(?:Infinity|(?:\\d+(?:\\.\\d*)?|\\.\\d+)(?:[eE][+-]?\\d+)?))/);
      const consumed = match ? match[0].length : 0;
      writePtr(endPtr, consumed ? ptr + consumed : ptr);
      return consumed ? Number(match[1]) : Number.NaN;
    },
    snprintf(buffer, size, formatPtr, valuePtr) {
      const format = readCString(formatPtr);
      const value = valuePtr ? readCString(valuePtr) : '';
      return writeCString(buffer, format.replace('%s', value), size);
    },
    vsnprintf(buffer, size, formatPtr) {
      return writeCString(buffer, readCString(formatPtr).replace(/%s/g, ''), size);
    },
    fprintf(_stream, formatPtr, valuePtr) {
      console.error(readCString(formatPtr).replace('%s', valuePtr ? readCString(valuePtr) : ''));
      return 0;
    },
    vfprintf(_stream, formatPtr) {
      console.error(readCString(formatPtr));
      return 0;
    },
    fwrite(ptr, size, count, _stream) {
      const length = size * count;
      const view = bytes().subarray(ptr, ptr + length);
      console.log(decoder.decode(view));
      return count;
    },
    fputc(ch, _stream) {
      console.log(String.fromCharCode(ch));
      return ch;
    },
    puts(ptr) {
      console.log(readCString(ptr));
      return 0;
    },
    printf(formatPtr, valuePtr) {
      console.log(readCString(formatPtr).replace('%s', valuePtr ? readCString(valuePtr) : ''));
      return 0;
    },
    putchar(ch) {
      console.log(String.fromCharCode(ch));
      return ch;
    },
    frexp(value, expPtr) {
      if (value === 0) {
        writeI32(expPtr, 0);
        return 0;
      }
      const exponent = Math.floor(Math.log2(Math.abs(value))) + 1;
      writeI32(expPtr, exponent);
      return value / 2 ** exponent;
    },
    scalbn(value, exponent) {
      return value * 2 ** exponent;
    },
    lrint(value) {
      return Math.round(value);
    },
    acosh: Math.acosh,
    asinh: Math.asinh,
    atanh: Math.atanh,
    localtime_r(timePtr, tmPtr) {
      const seconds = Number(new DataView(requireMemory().buffer).getBigInt64(timePtr, true));
      return fillTimeStruct(tmPtr, new Date(seconds * 1000));
    },
    __assert_fail(assertionPtr, filePtr, line, funcPtr) {
      throw new Error(`QuickJS assertion failed: ${readCString(assertionPtr)} at ${readCString(filePtr)}:${line} in ${readCString(funcPtr)}`);
    },
    abort() {
      throw new Error("QuickJS aborted execution");
    }
  });
}
"""
)
PY
