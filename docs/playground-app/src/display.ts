import type { RunResult } from './types';

export interface RunResultDisplay {
  hasError: boolean;
  resultText: string;
  stdoutText: string;
  stderrText: string;
  errorKindText: string;
  errorNameText: string;
  errorMessageText: string;
  errorStackText: string;
}

export function formatRunResultDisplay(runResult: RunResult | null): RunResultDisplay {
  if (!runResult) {
    return {
      hasError: false,
      resultText: 'Run code to inspect the evaluated result.',
      stdoutText: 'No stdout yet.',
      stderrText: 'No stderr yet.',
      errorKindText: 'None',
      errorNameText: 'None',
      errorMessageText: 'No error.',
      errorStackText: 'No stack.'
    };
  }

  return {
    hasError: Boolean(runResult.error),
    resultText: runResult.result ?? 'No result.',
    stdoutText: formatOutputLines(runResult.stdout, 'No stdout.'),
    stderrText: formatOutputLines(runResult.stderr, 'No stderr.'),
    errorKindText: runResult.error?.kind ?? 'None',
    errorNameText: runResult.error?.name ?? 'None',
    errorMessageText: runResult.error?.message ?? 'No error.',
    errorStackText: runResult.error?.stack ?? 'No stack.'
  };
}

function formatOutputLines(lines: string[], fallback: string): string {
  return lines.length ? lines.join('\n') : fallback;
}
