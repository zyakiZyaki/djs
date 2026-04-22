// Test: counter without effect
function counter() {
    function step(count) {
        return {
            inc: function () {
                return step(count + 1);
            }
        };
    }
    return step;
}

function test() {
    return counter()(0).inc()
}
test()
