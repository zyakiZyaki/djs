// Test: calling captured function in object method
function test() {
    function makeObj(fn) {
        return {
            call: function() {
                return fn(10);
            }
        };
    }
    function double(x) {
        return x * 2;
    }
    return makeObj(double).call()
}
test()
