// Test: function with expr stmt and return
function test() {
    function makeFn(fn) {
        return function() {
            fn(42);
            return 99;
        };
    }
    function log(x) {
        return x;
    }
    return makeFn(log)()
}
test()
