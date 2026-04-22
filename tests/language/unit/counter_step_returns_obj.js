// Test: step function returning object with method that calls eff
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
