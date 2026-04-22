// Counter effect pattern - using named function declarations
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

// Test: create counter, call inc twice, return final state
function test() {
    function logEffect(value) {
        return value;
    }
    counter(logEffect)(0).inc().inc()
}
test()
