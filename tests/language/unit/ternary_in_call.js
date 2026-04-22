// Ternary as function argument
function test() {
    function identity(x) { return x; }
    return identity(true ? 42 : 0);
}
test()
