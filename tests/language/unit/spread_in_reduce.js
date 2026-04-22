// Spread used with reduce
function test() {
    function flatten(arrs) {
        return arrs.reduce(function(acc, arr) {
            return acc.concat(arr);
        }, []);
    }
    return flatten([[1, 2], [3], [4, 5]]);
}
test()
