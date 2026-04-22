function test() {
    function makeAdder(x) {
        return function(y) {
            return x + y;
        };
    }
    function apply(fns, val) {
        return fns.reduce(function(acc, fn) {
            return fn(acc);
        }, val);
    }
    return apply([makeAdder(10), makeAdder(100), makeAdder(1000)], 0);
}
test()
