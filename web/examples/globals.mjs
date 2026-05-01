globalThis.counter = (globalThis.counter ?? 0) + 1;
export const result = `counter=${globalThis.counter}`;
