// True curry chain: add(5)(10)
function add(a) {
    return function(b) {
        return function(c) {
            return a + b + c;
        };
    };
}
add(10)(20)(30)
