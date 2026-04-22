function test() {
    function outer(a) {
        return function(b) {
            return function(c) {
                return a + b + c;
            };
        };
    }
    return outer(1)(2)(3);
}
test()
