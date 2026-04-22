// Test: counter effect - simple version
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
        console.log("effect:", value);
        return value;
    }
    return counter(logEffect)(0).inc()
}
test()
