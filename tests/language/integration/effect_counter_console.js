// Integration test: counter effect pattern with console.log tracing
function counter(eff) {
    return function step(count) {
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
    };
}

// Test: use console.log as effect, return final state
function test() {
    function makeCounter(eff) {
        return function step(count) {
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
        };
    }
    function logState(value) {
        console.log("state:", value);
        return value;
    }
    // Start at 5, go to 6, 7, 6, 7, 8
    makeCounter(logState)(5).inc().inc().dec().inc().inc()
}
test()
