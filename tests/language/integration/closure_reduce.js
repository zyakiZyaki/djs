function test() {
    function multiplier(n) {
        function sum(acc, x) {
            return acc + x * n;
        }
        return function(arr) {
            return arr.reduce(sum, 0);
        };
    }
    return multiplier(10)([1, 2, 3]);
}
test()
