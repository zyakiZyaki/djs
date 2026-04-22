// Test: counter pattern with just effect, no recursive step call
function counter(eff) {
    function step(count) {
        return {
            inc: function () {
                eff(count + 1);
                return count + 1;
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
