// Test: object with closure property
function test() {
    function makeObj() {
        return {
            func: function(x) {
                return x + 1;
            }
        };
    }
    return makeObj().func(10)
}
test()
