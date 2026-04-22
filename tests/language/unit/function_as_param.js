// Function passed as parameter and called
function test() {
    function apply(fn, val) {
        return fn(val);
    }
    function double(x) {
        return x * 2;
    }
    return apply(double, 21);
}
test()
