// Counter effect pattern - effect returns state object, chain operations
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

// Test: effect logs state, return final state via effect
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
    function stateLogger(value) {
        return value;
    }
    // Start at 100, inc to 101, dec to 100, inc to 101
    // The effect will be called with 101, 100, 101
    // Return the final step object (which contains inc/dec for next state 101)
    makeCounter(stateLogger)(100).inc().dec().inc()
}
test()
