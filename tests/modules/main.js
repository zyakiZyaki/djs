import { add, sub } from "./math";
import { fib } from "./fib";

function test() {
    return add(fib(10), sub(100, 50));
}
test()
