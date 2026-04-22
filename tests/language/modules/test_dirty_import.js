import { add, sub } from "../../modules/math";
import { fib } from "../../modules/fib";

function test() {
    return add(fib(10), sub(100, 50));
}
test()
