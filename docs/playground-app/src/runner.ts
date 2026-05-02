import type { RunResult, SampleMeta, SamplePayload } from './types';

interface PlaygroundBindings {
  default: () => Promise<unknown>;
  init: () => unknown;
  reset: () => unknown;
  list_samples: () => unknown;
  load_sample: (sampleId: string) => unknown;
  run: (source: string, sampleId?: string) => unknown;
}

let modulePromise: Promise<PlaygroundBindings> | null = null;

async function loadBindings(): Promise<PlaygroundBindings> {
  if (!modulePromise) {
    modulePromise = import('../../.vuepress/public/wasm/pkg/rquickjs_playground.js') as Promise<PlaygroundBindings>;
  }
  const bindings = await modulePromise;
  await bindings.default();
  return bindings;
}

export async function initPlayground(): Promise<void> {
  const bindings = await loadBindings();
  bindings.init();
}

export async function resetPlayground(): Promise<void> {
  const bindings = await loadBindings();
  bindings.reset();
}

export async function listSamples(): Promise<SampleMeta[]> {
  const bindings = await loadBindings();
  return bindings.list_samples() as SampleMeta[];
}

export async function loadSample(sampleId: string): Promise<SamplePayload> {
  const bindings = await loadBindings();
  return bindings.load_sample(sampleId) as SamplePayload;
}

export async function runSource(source: string, sampleId?: string): Promise<RunResult> {
  const bindings = await loadBindings();
  return bindings.run(source, sampleId) as RunResult;
}
