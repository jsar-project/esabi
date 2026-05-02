export interface SampleMeta {
  id: string;
  title: string;
  summary: string;
  mode: 'script' | 'module';
}

export interface SamplePayload extends SampleMeta {
  source: string;
  notes: string[];
}

export interface RunResult {
  ok: boolean;
  mode: 'script' | 'module';
  result?: string | null;
  stdout: string[];
  stderr: string[];
  error?: {
    kind: 'wasm-init' | 'execution' | 'internal';
    name: string;
    message: string;
    stack?: string | null;
  } | null;
}
