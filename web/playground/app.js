import init, {
  examples_json,
  run_playground,
  support_matrix_json,
} from "../dist/runner/boa_playground_runner.js";

const editor = document.querySelector("#editor");
const result = document.querySelector("#result");
const logs = document.querySelector("#logs");
const error = document.querySelector("#error");
const status = document.querySelector("#status");
const runButton = document.querySelector("#run");
const examplesNode = document.querySelector("#examples");
const supportNode = document.querySelector("#support");

let examples = [];
let activeExampleId = null;

boot().catch((err) => {
  setStatus(`Runner failed to load: ${err.message ?? err}`, true);
});

async function boot() {
  await init();
  examples = JSON.parse(examples_json());
  renderExamples();
  renderSupport(JSON.parse(support_matrix_json()));
  activateExample(examples[0]?.id ?? null);
  setStatus("Runner ready.", false);
  runButton.disabled = false;
}

runButton.addEventListener("click", () => {
  const output = JSON.parse(run_playground(editor.value));
  renderOutput(output);
  setStatus(output.ok ? "Execution completed." : "Execution failed.", !output.ok);
});

function renderExamples() {
  examplesNode.innerHTML = "";
  for (const example of examples) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = example.title;
    button.addEventListener("click", () => activateExample(example.id));
    examplesNode.appendChild(button);
  }
}

function activateExample(id) {
  activeExampleId = id;
  const example = examples.find((item) => item.id === id);
  if (!example) {
    return;
  }

  editor.value = example.code.trim();
  for (const button of examplesNode.querySelectorAll("button")) {
    button.classList.toggle("active", button.textContent === example.title);
  }
  renderOutput({
    ok: true,
    result: example.description,
    logs: [],
    error: null,
  });
}

function renderSupport(items) {
  supportNode.innerHTML = "";
  for (const item of items) {
    const card = document.createElement("article");
    card.className = "support-item";
    card.innerHTML = `
      <header>
        <strong>${escapeHtml(item.name)}</strong>
        <span class="badge ${escapeHtml(item.status)}">${escapeHtml(item.status)}</span>
      </header>
      <p>${escapeHtml(item.detail)}</p>
    `;
    supportNode.appendChild(card);
  }
}

function renderOutput(output) {
  result.textContent = output.result ?? "No exported `result` value.";
  logs.textContent = output.logs?.length ? output.logs.join("\n") : "No logs.";
  error.textContent = output.error ?? "No errors.";
}

function setStatus(message, isError) {
  status.textContent = message;
  status.style.color = isError ? "var(--danger)" : "var(--muted)";
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
