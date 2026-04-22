// Callback chain with closures — pure functions
function test() {
    function applyTwice(fn, val) {
        return fn(fn(val));
    }
    return applyTwice(function(x) { return x * 2; }, 3);
}
test()
