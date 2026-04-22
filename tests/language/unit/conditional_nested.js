// Deeply nested conditionals
function test() {
    function classify(n) {
        return n < 0
            ? 0 - 1
            : n == 0
                ? 0
                : n < 10
                    ? 1
                    : n < 100
                        ? 2
                        : 3;
    }
    return [classify(0 - 5), classify(0), classify(5), classify(50), classify(500)];
}
test()
