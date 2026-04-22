// Test: counter with effect call
function counter(eff) {
    function step(count) {
        return {
            inc: function () {
                eff(count + 1);
                return step(count + 1);
            }
        };
    }
    return step;
}

function test() {
    function logEffect(value) {
        return value;
    }
    return counter(logEffect)(0).inc()
}
test()
