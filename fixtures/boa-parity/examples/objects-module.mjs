import { fixtureBox, makePoint } from "demo:objects";

const point = makePoint(2, 3);

export const result = `${fixtureBox.label}|${point.sum}|${fixtureBox.nested.ok}`;
