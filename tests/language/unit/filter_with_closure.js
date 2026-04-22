// filter using closure-captured value
function test() {
    function makeFilter(min) {
        return function(arr) {
            return arr.filter(function(x) { return x >= min; });
        };
    }
    return makeFilter(5)([1, 10, 3, 8, 2]);
}
test()
