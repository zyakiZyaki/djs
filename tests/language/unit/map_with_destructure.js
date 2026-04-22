// map with destructuring callback
function test() {
    return [{val: 1}, {val: 2}, {val: 3}].map(function(x) {
        return x.val * 10;
    });
}
test()
