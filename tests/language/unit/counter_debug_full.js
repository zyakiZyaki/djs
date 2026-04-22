// Debug: check what eff is inside inc method
function counter(eff) {
    function step(count) {
        return {
            inc: function () {
                console.log("eff type:", eff);
                console.log("count:", count);
                eff(count + 1);
                return step(count + 1);
            }
        };
    }
    return step;
}

function test() {
    function logEffect(value) {
        console.log("effect called:", value);
        return value;
    }
    return counter(logEffect)(0).inc()
}
test()
