import { resolvedLabel } from "demo:promises";

export let result = "pending";

resolvedLabel(7).then((value) => {
  result = `done:${value}`;
});
