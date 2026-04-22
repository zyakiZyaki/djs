// JSON.parse + array methods
function test() {
    return JSON.parse('[1,2,3,4,5]').map(function(x) { return x * 2; });
}
test()
