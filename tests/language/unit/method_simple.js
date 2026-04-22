// Test: very simple chained call with return
function test() {
    function makeCounter() {
        return {
            inc: function () {
                return 42;
            }
        };
    }
    return makeCounter().inc()
}
test()
