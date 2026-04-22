// Boolean edge cases
function test() {
    return [
        true && true,
        false && true,
        true || false,
        false || false,
        (5 > 3) && (10 < 20),
        (5 < 3) || (10 > 20)
    ];
}
test()
