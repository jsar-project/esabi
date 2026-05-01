import { failWith } from "demo:errors";

let message = "ok";

try {
  failWith("fixture boom");
} catch (error) {
  message = String(error.message ?? error);
}

export const result = message;
