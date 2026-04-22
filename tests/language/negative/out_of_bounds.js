// ERROR: array index out of bounds should not crash
function test() {
    return [1][999];
}
test()
