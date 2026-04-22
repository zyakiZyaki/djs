// Test: captured function call in zero-param object method
function test() {
    function makeObj(eff) {
        return {
            method: function() {
                eff(42);
                return 99;
            }
        };
    }
    function log(x) {
        return x;
    }
    return makeObj(log).method()
}
test()
