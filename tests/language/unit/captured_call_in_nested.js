// Test: simpler - captured function call in function (not object method)
function test() {
    function outer(eff) {
        function inner(count) {
            eff(count + 1);
            return count + 1;
        }
        return inner;
    }
    function log(x) {
        return x;
    }
    return outer(log)(0)
}
test()
