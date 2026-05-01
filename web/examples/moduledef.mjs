import { add, answer } from "demo:math";
import { log } from "demo:console";

const total = add(8, 10) + answer;
log("demo:math total =", total);

export const result = `total=${total}`;
