// Closure that returns array operations
function test() {
    function makeFilter(threshold) {
        return function(arr) {
            return arr.filter(function(x) { return x > threshold; });
        };
    }
    return makeFilter(5)([1, 3, 7, 10, 2, 8]);
}
test()
