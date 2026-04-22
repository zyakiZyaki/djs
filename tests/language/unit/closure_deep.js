// Deeply nested closures (3 levels)
function test() {
    function level1(a) {
        return function level2(b) {
            return function level3(c) {
                return a + b + c;
            };
        };
    }
    return level1(1)(2)(3);
}
test()
