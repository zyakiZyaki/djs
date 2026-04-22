// Test: closure capturing and calling function
function test() {
    function outer(fn) {
        return function() {
            fn(42);
            return 99;
        };
    }
    function identity(x) {
        return x;
    }
    return outer(identity)()
}
test()
