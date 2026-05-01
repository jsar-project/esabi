import { invokeTimer, readUrl, upper } from "demo:functions";

const upperResult = upper("boa");
const readUrlResult = readUrl({});

let timerValue = "";
const invokedDelay = invokeTimer((message) => {
  timerValue = message;
}, 12);

export const result = [
  upperResult,
  readUrlResult,
  timerValue,
  String(invokedDelay),
  String(globalThis.lastDelay),
].join("|");
