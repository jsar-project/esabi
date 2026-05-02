<template>
  <section class="playground-shell">
    <header class="ide-bar">
      <div class="ide-bar-controls">
        <label class="sample-select-wrap">
          <span class="sample-select-label">Sample</span>
          <select
            :value="selectedSampleId"
            class="sample-select"
            :disabled="busy"
            @change="onSampleChange"
          >
            <option v-for="sample in samples" :key="sample.id" :value="sample.id">
              {{ sample.title }}
            </option>
          </select>
        </label>
        <div class="toolbar-actions">
          <button type="button" class="ghost" :disabled="busy" @click="resetRuntime">Reset</button>
          <button type="button" class="primary" :disabled="busy" @click="runCurrent">
            {{ busy ? 'Running...' : 'Run' }}
          </button>
        </div>
      </div>
      <p class="ide-bar-status">{{ status }}</p>
    </header>

    <main class="workspace">
      <div class="workspace-main">
        <section class="editor-pane">
          <div class="pane-head">
            <p class="pane-title">Source</p>
          </div>

          <textarea v-model="source" class="editor" spellcheck="false" />

          <ul v-if="activeSample?.notes?.length" class="notes">
            <li v-for="note in activeSample.notes" :key="note">{{ note }}</li>
          </ul>
        </section>

        <section class="output-pane">
          <div class="pane-head">
            <p class="pane-title">Output</p>
          </div>

          <pre class="output-view" :class="{ error: Boolean(runResult?.error) }">{{ outputText }}</pre>
        </section>
      </div>
    </main>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { initPlayground, listSamples, loadSample, resetPlayground, runSource } from '../../playground-app/src/runner';
import type { RunResult, SampleMeta, SamplePayload } from '../../playground-app/src/types';

const busy = ref(false);
const status = ref('Loading wasm runtime...');
const samples = ref<SampleMeta[]>([]);
const selectedSampleId = ref('hello-world');
const activeSample = ref<SamplePayload | null>(null);
const source = ref('');
const runResult = ref<RunResult | null>(null);

const stdoutText = computed(() => (runResult.value?.stdout.length ? runResult.value.stdout.join('\n') : 'No stdout yet.'));
const errorText = computed(() => {
  const error = runResult.value?.error;
  if (!error) {
    return 'No errors.';
  }
  return [error.kind, `${error.name}: ${error.message}`, error.stack ?? ''].filter(Boolean).join('\n\n');
});
const outputText = computed(() => {
  const error = runResult.value?.error;
  if (error) {
    return errorText.value;
  }

  const sections: string[] = [];
  const result = runResult.value?.result;
  const stdout = runResult.value?.stdout?.length ? runResult.value.stdout.join('\n') : '';
  const stderr = runResult.value?.stderr?.length ? runResult.value.stderr.join('\n') : '';

  if (result) {
    sections.push(result);
  }
  if (stdout) {
    sections.push(`stdout:\n${stdout}`);
  }
  if (stderr) {
    sections.push(`stderr:\n${stderr}`);
  }

  return sections.length ? sections.join('\n') : '>';
});

function onSampleChange(event: Event): void {
  const target = event.target as HTMLSelectElement | null;
  if (!target?.value) {
    return;
  }
  void selectSample(target.value);
}

async function hydrateSamples(): Promise<void> {
  samples.value = await listSamples();
  if (!samples.value.length) {
    return;
  }
  if (!samples.value.some((sample) => sample.id === selectedSampleId.value)) {
    selectedSampleId.value = samples.value[0].id;
  }
  await selectSample(selectedSampleId.value);
}

async function selectSample(sampleId: string): Promise<void> {
  busy.value = true;
  status.value = `Loading sample: ${sampleId}`;
  try {
    const sample = await loadSample(sampleId);
    selectedSampleId.value = sample.id;
    activeSample.value = sample;
    source.value = sample.source;
    runResult.value = null;
    status.value = `Ready: ${sample.title}`;
  } finally {
    busy.value = false;
  }
}

async function runCurrent(): Promise<void> {
  busy.value = true;
  status.value = 'Executing sample...';
  try {
    runResult.value = await runSource(source.value, selectedSampleId.value);
    status.value = runResult.value.ok ? 'Execution complete' : 'Execution finished with errors';
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    runResult.value = {
      ok: false,
      mode: activeSample.value?.mode ?? 'script',
      stdout: [],
      stderr: [],
      result: null,
      error: {
        kind: 'wasm-init',
        name: 'PlaygroundBootstrapError',
        message,
        stack: error instanceof Error ? error.stack ?? null : null
      }
    };
    status.value = 'Playground bootstrap failed';
  } finally {
    busy.value = false;
  }
}

async function resetRuntime(): Promise<void> {
  busy.value = true;
  status.value = 'Resetting runtime...';
  try {
    await resetPlayground();
    await selectSample(selectedSampleId.value);
    status.value = 'Runtime reset';
  } finally {
    busy.value = false;
  }
}

onMounted(async () => {
  try {
    await initPlayground();
    await hydrateSamples();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    status.value = `Failed to initialize playground: ${message}`;
  }
});
</script>

<style scoped>
.playground-shell {
  display: grid;
  gap: 0.6rem;
  width: 100%;
  min-height: calc(100vh - 6.5rem);
}

.ide-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.4rem 0.75rem;
}

.ide-bar-controls {
  display: flex;
  align-items: end;
  gap: 0.85rem;
  flex: 1 1 auto;
  justify-content: flex-start;
}

.ide-bar-status {
  margin: 0;
  flex: 0 0 320px;
  color: #94a3b8;
  font-size: 0.92rem;
  line-height: 1.4;
  text-align: right;
}

.pane-title,
.sample-select-label,
.notes {
  color: #cbd5e1;
}

.workspace {
  display: grid;
  gap: 0;
}

.pane-title {
  margin: 0;
  font-size: 0.82rem;
  font-weight: 700;
  color: #e2e8f0;
}

.sample-select-wrap {
  display: grid;
  gap: 0.35rem;
  min-width: 240px;
}

.sample-select-label {
  font-size: 0.78rem;
}

.sample-select {
  appearance: none;
  width: 100%;
  padding: 0.7rem 0.85rem;
  border-radius: 8px;
  border: 1px solid rgba(148, 163, 184, 0.18);
  background: #020617;
  color: #f8fafc;
  font: inherit;
}

.workspace-main {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 300px;
  gap: 0.5rem;
  align-items: stretch;
  min-height: clamp(540px, calc(100vh - 10.5rem), 820px);
}

.editor-pane,
.output-pane {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  min-height: 100%;
  min-width: 0;
  border: 1px solid rgba(148, 163, 184, 0.14);
  background: rgba(15, 23, 42, 0.9);
}

.editor-pane {
  padding: 0;
}

.output-pane {
  padding: 0;
  min-height: 100%;
}

.pane-head {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem 0.9rem;
  border-bottom: 1px solid rgba(148, 163, 184, 0.12);
}

.toolbar-actions {
  display: flex;
  gap: 0.75rem;
  flex: 0 0 auto;
}

button {
  cursor: pointer;
}

.primary,
.ghost {
  border-radius: 8px;
  padding: 0.7rem 0.95rem;
  border: 1px solid rgba(148, 163, 184, 0.16);
  color: #f8fafc;
  font-weight: 600;
}

.primary {
  background: #2563eb;
}

.ghost {
  background: rgba(15, 23, 42, 0.6);
}

.editor {
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  min-height: 0;
  min-width: 0;
  height: 100%;
  max-height: 100%;
  resize: none;
  border: 0;
  border-radius: 0;
  display: block;
  padding: 0.9rem 1rem;
  font: 500 0.96rem/1.72 'SFMono-Regular', 'Menlo', monospace;
  color: #f8fafc;
  background: #020617;
  overflow: auto;
  white-space: pre;
}

.notes {
  margin: 0;
  padding: 0.75rem 1.6rem 0.9rem;
  line-height: 1.55;
  border-top: 1px solid rgba(148, 163, 184, 0.12);
  background: rgba(2, 6, 23, 0.32);
}

.output-pane {
  display: grid;
  overflow: hidden;
}

.output-view {
  box-sizing: border-box;
  height: 100%;
  max-height: 100%;
  min-height: 0;
  margin: 0;
  padding: 0.9rem;
  background: #020617;
  color: #e2e8f0;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.output-view.error {
  color: #fca5a5;
}

@media (max-width: 1240px) {
  .ide-bar {
    align-items: stretch;
  }

  .ide-bar-controls {
    justify-content: flex-start;
  }

  .workspace-main {
    grid-template-columns: 1fr;
    min-height: auto;
  }
}

@media (max-width: 960px) {
  .playground-shell {
    min-height: auto;
  }

  .ide-bar,
  .ide-bar-controls,
  .toolbar-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .ide-bar-status {
    flex: 0 0 auto;
    text-align: left;
  }

  .editor {
    height: 460px;
  }
}
</style>
