import { fib } from "./fib";

// Pure function: dependency passed as parameter
function test(f) {
    return f(10);
}
test(fib)
