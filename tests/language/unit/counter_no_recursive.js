// Test: object method with expr stmt but no recursive call
function test() {
    function makeCounter(eff) {
        function step(count) {
            return {
                inc: function() {
                    eff(count + 1);
                    return count + 1;
                }
            };
        }
        return step;
    }
    function log(x) {
        return x;
    }
    return makeCounter(log)(0).inc()
}
test()
