// Spread args into 3-param function
function test() {
    function sum(a, b, c) {
        return a + b + c;
    }
    return sum(...[10, 20, 30]);
}
test()
