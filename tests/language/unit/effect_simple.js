// Simple test: call effect, return different value
function test() {
    function withEffect(eff) {
        return function() {
            eff(42);
            return 100;
        };
    }
    function log(x) {
        return x;
    }
    withEffect(log)()
}
test()
