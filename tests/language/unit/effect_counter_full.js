// Counter effect pattern - full working version
function counter(eff) {
    function step(count) {
        return {
            inc: function () {
                eff(count + 1);
                return step(count + 1);
            },
            dec: function () {
                eff(count - 1);
                return step(count - 1);
            }
        };
    }
    return step;
}

// Test: counter with effect chain
function test() {
    function logEffect(value) {
        console.log("effect:", value);
        return value;
    }
    return counter(logEffect)(0).inc().inc().dec().inc()
}
test()
