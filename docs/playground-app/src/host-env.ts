import { installRquickjsEnv } from '../../.vuepress/public/wasm/pkg/rquickjs_host.js';

const env = installRquickjsEnv({});

export const malloc = (...args: [number]) => env.malloc(...args);
export const calloc = (...args: [number, number]) => env.calloc(...args);
export const realloc = (...args: [number, number]) => env.realloc(...args);
export const free = (...args: [number]) => env.free(...args);
export const strlen = (...args: [number]) => env.strlen(...args);
export const memchr = (...args: [number, number, number]) => env.memchr(...args);
export const strchr = (...args: [number, number]) => env.strchr(...args);
export const strrchr = (...args: [number, number]) => env.strrchr(...args);
export const strcmp = (...args: [number, number]) => env.strcmp(...args);
export const strncmp = (...args: [number, number, number]) => env.strncmp(...args);
export const strtod = (...args: [number, number]) => env.strtod(...args);
export const snprintf = (...args: [number, number, number, number]) => env.snprintf(...args);
export const vsnprintf = (...args: [number, number, number]) => env.vsnprintf(...args);
export const fprintf = (...args: [number, number, number]) => env.fprintf(...args);
export const vfprintf = (...args: [number, number]) => env.vfprintf(...args);
export const fwrite = (...args: [number, number, number, number]) => env.fwrite(...args);
export const fputc = (...args: [number, number]) => env.fputc(...args);
export const puts = (...args: [number]) => env.puts(...args);
export const printf = (...args: [number, number]) => env.printf(...args);
export const putchar = (...args: [number]) => env.putchar(...args);
export const frexp = (...args: [number, number]) => env.frexp(...args);
export const scalbn = (...args: [number, number]) => env.scalbn(...args);
export const lrint = (...args: [number]) => env.lrint(...args);
export const acosh = (...args: [number]) => env.acosh(...args);
export const asinh = (...args: [number]) => env.asinh(...args);
export const atanh = (...args: [number]) => env.atanh(...args);
export const localtime_r = (...args: [number, number]) => env.localtime_r(...args);
export const __assert_fail = (...args: [number, number, number, number]) =>
  env.__assert_fail(...args);
export const abort = (...args: []) => env.abort(...args);
