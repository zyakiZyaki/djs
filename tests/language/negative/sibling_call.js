// ERROR: global function calling sibling global function
function outer() {
    function helper() { return 1; }
    function inner() {
        return helper();
    }
    return inner();
}
outer()
