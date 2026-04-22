// reduce with rest-like pattern
function test() {
    function sumAll(...vals) {
        return vals.reduce(function(acc, x) {
            return acc + x;
        }, 0);
    }
    return sumAll(1, 2, 3, 4, 5);
}
test()
