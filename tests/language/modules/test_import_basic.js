import { add, mul } from "./ops";

// Pure function: all dependencies passed as parameters
function test(a, b) {
    return b(a(10, 20), 3);
}
test(add, mul)
