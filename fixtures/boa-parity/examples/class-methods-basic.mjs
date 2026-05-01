import { Greeter, makeGreeter } from "demo:class-basic";

const greeter = makeGreeter("boa");
greeter.label = "buddy";
const constructed = new Greeter("again");
const kindDesc = Object.getOwnPropertyDescriptor(Greeter.prototype, "kind");
export const result = `${greeter.greet()}|${greeter.label}|${constructed.greet()}|${kindDesc.enumerable}`;
