// Test: method call with logging
function test() {
    function makeCounter() {
        return {
            inc: function () {
                console.log("inc called");
                return 42;
            }
        };
    }
    makeCounter().inc()
}
test()
