// Integration test: counter effects processed through chained calls
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

// Test: apply sequence of operations
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
    function runOps() {
        function logEff(value) {
            return value;
        }
        makeCounter(logEff)(0).inc().inc().dec().inc().inc()
    }
    runOps()
}
test()
