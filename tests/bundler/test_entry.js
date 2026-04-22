// Bundler entry point - imports math module
import { add, mul } from "./ops";

function test() {
    return mul(add(10, 20), 3);
}
test()
