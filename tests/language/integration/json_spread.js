// JSON.stringify + spread
function test() {
    function merge(...arrs) {
        return arrs.reduce(function(acc, arr) {
            return acc.concat(arr);
        }, []);
    }
    return JSON.stringify(merge([1, 2], [3]));
}
test()
