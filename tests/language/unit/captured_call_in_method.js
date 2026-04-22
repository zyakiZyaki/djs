// Test: captured function call in object method
function test() {
    function makeObj(eff) {
        return {
            method: function(count) {
                eff(count + 1);
                return count + 1;
            }
        };
    }
    function log(x) {
        return x;
    }
    return makeObj(log).method(0)
}
test()
