// Counter effect pattern - dec operation and mixed chains
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

// Test: mixed inc/dec chain
function test() {
    function logEffect(value) {
        return value;
    }
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
    // Start at 0, inc to 1, inc to 2, dec to 1, inc to 2
    makeCounter(logEffect)(0).inc().inc().dec().inc()
}
test()
